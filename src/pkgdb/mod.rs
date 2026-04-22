//! Package metadata cache layer.
//!
//! Responsibilities:
//! - keep a local SQLite cache of package metadata
//! - refresh from apt/dpkg data
//! - apply optional runtime section mapping overrides
use anyhow::{Context, Result};
use chrono::{DateTime, Local};
use rusqlite::{params, Connection, Row};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::process::Command;
use tracing::{debug, info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Package metadata row stored in the local cache.
pub struct Package {
    /// Debian/apt package name.
    pub name: String,
    /// Candidate version from available repositories.
    pub version: String,
    /// Whether package is currently installed.
    pub installed: bool,
    /// Installed version when package is installed.
    pub installed_version: Option<String>,
    /// Package size in bytes (when available).
    pub size: Option<u64>,
    /// Human-readable package summary/description.
    pub description: String,
    /// Parsed dependency package names.
    pub depends: Vec<String>,
    /// Debian priority field.
    pub priority: String,
    /// Logical section used by muxitude grouping.
    pub section: String,
    /// Package maintainer string.
    pub maintainer: String,
    /// Target architecture string.
    pub architecture: String,
    /// Upstream homepage URL when available.
    pub homepage: Option<String>,
}

#[derive(Debug, Clone)]
/// SQLite-backed package cache and refresh driver.
pub struct PackageCache {
    db_path: PathBuf,
    section_mappings_merge_path: Option<PathBuf>,
}

mod parser;

impl PackageCache {
    /// Normalize a section name to muxitude canonical form.
    ///
    /// Component prefixes like `contrib/sound` are reduced to `sound`.
    ///
    pub(super) fn canonical_section(section: &str) -> String {
        // Debian indexes may include component-prefixed sections such as
        // "contrib/sound" or "non-free/libs". Aptitude-style section trees
        // should use the actual section ("sound", "libs").
        section
            .trim()
            .to_ascii_lowercase()
            .rsplit('/')
            .next()
            .unwrap_or("uncategorized")
            .trim()
            .to_string()
    }

    /// Parse `package=section` mapping lines into a hash map.
    ///
    /// Invalid lines are skipped with a warning.
    ///
    pub(super) fn parse_section_mappings(raw: &str, source: &str) -> HashMap<String, String> {
        // Accept simple key=value lines; skip invalid entries but keep loading.
        let mut mappings = HashMap::new();

        for (idx, line) in raw.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            let Some((pkg, section)) = trimmed.split_once('=') else {
                warn!(
                    "Invalid section mapping at {}:{}: {}",
                    source,
                    idx + 1,
                    trimmed
                );
                continue;
            };

            let package = pkg.trim().to_ascii_lowercase();
            let section_name = Self::canonical_section(section);
            if package.is_empty() || section_name.is_empty() {
                warn!(
                    "Invalid section mapping at {}:{}: {}",
                    source,
                    idx + 1,
                    trimmed
                );
                continue;
            }

            mappings.insert(package, section_name);
        }

        mappings
    }

    /// Load embedded section mappings and apply optional runtime overrides.
    ///
    pub(super) fn load_section_mappings(&self) -> Result<HashMap<String, String>> {
        // Start from embedded mappings, then apply runtime overrides.
        let mut mappings = Self::parse_section_mappings(
            include_str!("../../res/section-mappings.txt"),
            "embedded",
        );

        if let Some(path) = &self.section_mappings_merge_path {
            let raw = std::fs::read_to_string(path).with_context(|| {
                format!(
                    "Failed to read section mappings merge file: {}",
                    path.display()
                )
            })?;
            let merged = Self::parse_section_mappings(&raw, &path.display().to_string());
            mappings.extend(merged);
        }

        Ok(mappings)
    }

    /// Create a package cache instance and initialize its database.
    ///
    pub fn new_with_section_mappings_merge(
        section_mappings_merge_path: Option<PathBuf>,
    ) -> Result<Self> {
        // Cache lives under XDG/OS cache directory.
        let cache_dir = dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("muxitude");
        std::fs::create_dir_all(&cache_dir)?;

        let db_path = cache_dir.join("packages.db");

        let cache = Self {
            db_path,
            section_mappings_merge_path,
        };

        cache.init_database()?;
        Ok(cache)
    }

    /// Create/migrate SQLite schema used by the cache.
    ///
    pub(super) fn init_database(&self) -> Result<()> {
        // Single table cache with last-updated timestamps for stale checks.
        let conn = Connection::open(&self.db_path)?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS packages (
                name TEXT PRIMARY KEY,
                version TEXT,
                installed INTEGER,
                installed_version TEXT,
                size INTEGER,
                description TEXT,
                depends TEXT,
                priority TEXT,
                section TEXT,
                maintainer TEXT,
                architecture TEXT,
                homepage TEXT,
                first_seen TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                last_updated TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;

        // Migration for existing databases created before first_seen existed.
        if let Err(e) = conn.execute(
            "ALTER TABLE packages ADD COLUMN first_seen TIMESTAMP DEFAULT CURRENT_TIMESTAMP",
            [],
        ) {
            let msg = e.to_string();
            if !msg.contains("duplicate column name") {
                return Err(e.into());
            }
        }

        conn.execute("CREATE INDEX IF NOT EXISTS idx_name ON packages(name)", [])?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_installed ON packages(installed)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_section ON packages(section)",
            [],
        )?;

        Ok(())
    }

    /// Refresh package cache when it is older than ~1 hour.
    ///
    pub fn refresh_if_needed(&self) -> Result<()> {
        // Avoid refreshing on every startup; refresh roughly hourly.
        let conn = Connection::open(&self.db_path)?;
        let mut stmt = conn.prepare("SELECT MAX(last_updated) FROM packages")?;
        let last_update: Option<String> = stmt.query_row([], |row| row.get(0)).ok().flatten();

        let needs_refresh = match last_update {
            Some(date) => {
                let last = DateTime::parse_from_str(&date, "%Y-%m-%d %H:%M:%S")
                    .map(|dt| dt.with_timezone(&Local))
                    .unwrap_or_else(|_| Local::now());
                let now = Local::now();
                (now - last).num_hours() >= 1
            }
            None => true,
        };

        if needs_refresh {
            info!("Cache is stale, refreshing...");
            self.refresh()?;
        } else {
            debug!("Cache is fresh");
        }

        Ok(())
    }

    /// Run `apt update` and rebuild cache contents.
    ///
    pub fn refresh(&self) -> Result<()> {
        // Refresh apt metadata first, then rebuild local cache records.
        info!("Running apt update...");
        let update_output = Command::new("apt")
            .arg("update")
            .output()
            .context("Failed to run apt update")?;

        if !update_output.status.success() {
            let stderr = String::from_utf8_lossy(&update_output.stderr);
            warn!("apt update failed: {}", stderr.trim());
        }

        self.refresh_after_update()
    }

    /// Rebuild cache content after apt metadata is updated.
    ///
    pub fn refresh_after_update(&self) -> Result<()> {
        // Rebuild cache from installed + available package metadata.
        let installed = self.get_installed_packages()?;
        let available = self.get_available_packages_improved(&installed)?;
        let is_bootstrap = self.is_cache_empty()?;
        self.store_to_db(&available, is_bootstrap)?;

        info!("Cache refreshed: {} packages total", available.len());
        Ok(())
    }

    /// Return whether the cache currently has no package rows.
    ///
    pub(super) fn is_cache_empty(&self) -> Result<bool> {
        let conn = Connection::open(&self.db_path)?;
        let mut stmt = conn.prepare("SELECT COUNT(*) FROM packages")?;
        let count: i64 = stmt.query_row([], |row| row.get(0))?;
        Ok(count == 0)
    }

    /// Return installed package/version map parsed from `dpkg -l`.
    ///
    pub(super) fn get_installed_packages(&self) -> Result<HashMap<String, String>> {
        let output = Command::new("dpkg")
            .args(["-l"])
            .output()
            .context("Failed to run dpkg -l")?;

        let stdout = String::from_utf8(output.stdout)?;
        let mut installed = HashMap::new();

        for line in stdout.lines() {
            if line.starts_with("ii") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 3 {
                    let name = parts[1].to_string();
                    let version = parts[2].to_string();
                    installed.insert(name, version);
                }
            }
        }

        debug!("Found {} installed packages", installed.len());
        Ok(installed)
    }

    /// Upsert package records into SQLite cache.
    ///
    /// When `bootstrap` is true, marks `first_seen` far in the past so that
    /// first refresh does not classify existing rows as "new".
    ///
    pub(super) fn store_to_db(&self, packages: &[Package], bootstrap: bool) -> Result<()> {
        let mut conn = Connection::open(&self.db_path)?;
        let tx = conn.transaction()?;

        for pkg in packages {
            tx.execute(
                "INSERT INTO packages
                 (name, version, installed, installed_version, size, description, depends,
                  priority, section, maintainer, architecture, homepage, first_seen, last_updated)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12,
                         CASE WHEN ?13 THEN datetime('now','-3650 days') ELSE CURRENT_TIMESTAMP END,
                         CURRENT_TIMESTAMP)
                 ON CONFLICT(name) DO UPDATE SET
                   version = excluded.version,
                   installed = excluded.installed,
                   installed_version = excluded.installed_version,
                   size = excluded.size,
                   description = excluded.description,
                   depends = excluded.depends,
                   priority = excluded.priority,
                   section = excluded.section,
                   maintainer = excluded.maintainer,
                   architecture = excluded.architecture,
                   homepage = excluded.homepage,
                   last_updated = CURRENT_TIMESTAMP",
                params![
                    pkg.name,
                    pkg.version,
                    pkg.installed,
                    pkg.installed_version,
                    pkg.size,
                    pkg.description,
                    serde_json::to_string(&pkg.depends)?,
                    pkg.priority,
                    pkg.section,
                    pkg.maintainer,
                    pkg.architecture,
                    pkg.homepage,
                    bootstrap,
                ],
            )?;
        }

        tx.commit()?;
        info!("Stored {} packages to database", packages.len());
        Ok(())
    }

    /// Return all cached package rows sorted by name.
    ///
    pub fn get_all(&self) -> Result<Vec<Package>> {
        let conn = Connection::open(&self.db_path)?;
        let mut stmt = conn.prepare("SELECT * FROM packages ORDER BY name")?;
        let packages = stmt
            .query_map([], |row| self.row_to_package(row))?
            .collect::<rusqlite::Result<Vec<Package>>>()?;
        Ok(packages)
    }

    /// Return currently uninstalled packages seen within `days`.
    ///
    pub fn get_new_packages(&self, days: i64) -> Result<Vec<Package>> {
        let conn = Connection::open(&self.db_path)?;
        let mut stmt = conn.prepare(
            "SELECT * FROM packages
             WHERE installed = 0
               AND datetime(first_seen) >= datetime('now', ?1)
             ORDER BY name",
        )?;
        let window = format!("-{} days", days.max(0));
        let packages = stmt
            .query_map([window], |row| self.row_to_package(row))?
            .collect::<rusqlite::Result<Vec<Package>>>()?;
        Ok(packages)
    }

    /// Clear "new package" markers for uninstalled packages.
    ///
    pub fn forget_new_packages(&self) -> Result<()> {
        let conn = Connection::open(&self.db_path)?;
        conn.execute(
            "UPDATE packages
             SET first_seen = datetime('now','-3650 days'),
                 last_updated = CURRENT_TIMESTAMP
             WHERE installed = 0",
            [],
        )?;
        Ok(())
    }

    /// Return currently upgradable packages as full cache rows.
    ///
    pub fn get_upgradable(&self) -> Result<Vec<Package>> {
        let output = Command::new("apt")
            .args(["list", "--upgradable"])
            .output()
            .context("Failed to run apt list --upgradable")?;

        let stdout = String::from_utf8(output.stdout)?;
        let mut names: HashSet<String> = HashSet::new();
        for line in stdout.lines() {
            if line.is_empty() || line.starts_with("Listing...") {
                continue;
            }
            if let Some((name, _)) = line.split_once('/') {
                names.insert(name.to_string());
            }
        }

        if names.is_empty() {
            return Ok(Vec::new());
        }

        let all = self.get_all()?;
        Ok(all
            .into_iter()
            .filter(|p| names.contains(&p.name))
            .collect())
    }

    /// Convert a SQLite row into a `Package` struct.
    ///
    pub(super) fn row_to_package(&self, row: &Row) -> rusqlite::Result<Package> {
        let depends_str: String = row.get("depends")?;
        let depends = serde_json::from_str(&depends_str).unwrap_or_default();

        Ok(Package {
            name: row.get("name")?,
            version: row.get("version")?,
            installed: row.get::<_, i32>("installed")? == 1,
            installed_version: row.get("installed_version")?,
            size: row.get("size")?,
            description: row.get("description")?,
            depends,
            priority: row.get("priority")?,
            section: row.get("section")?,
            maintainer: row.get("maintainer")?,
            architecture: row.get("architecture")?,
            homepage: row.get("homepage")?,
        })
    }
}
