//! Parsing helpers for package metadata ingestion.
use super::*;

impl PackageCache {
    /// Parse available packages from `apt-cache dumpavail`.
    ///
    /// Uses section mappings (embedded + optional merge file) to assign
    /// muxitude section names.
    ///
    pub(super) fn get_available_packages_improved(
        &self,
        installed: &HashMap<String, String>,
    ) -> Result<Vec<Package>> {
        // Parse `apt-cache dumpavail` control stanzas into cache Package rows.
        let section_mappings = self.load_section_mappings()?;
        let has_leading_ws = |s: &str| s.chars().next().map(|c| c.is_whitespace()).unwrap_or(false);
        let field_value = |line: &str, key: &str| -> Option<String> {
            let (k, v) = line.split_once(':')?;
            if k.trim().eq_ignore_ascii_case(key) {
                Some(v.trim_start().to_string())
            } else {
                None
            }
        };

        let output = Command::new("apt-cache")
            .args(["dumpavail"])
            .output()
            .context("Failed to run apt-cache dumpavail")?;

        let stdout = String::from_utf8(output.stdout)?;
        let mut packages = Vec::new();
        let mut current_pkg: Option<Package> = None;
        let mut current_desc = String::new();
        let mut in_description = false;

        for line in stdout.lines() {
            // Empty line closes a package stanza.
            if line.is_empty() {
                if let Some(mut pkg) = current_pkg.take() {
                    pkg.section = section_mappings
                        .get(&pkg.name.to_ascii_lowercase())
                        .cloned()
                        .unwrap_or_else(|| "uncategorized".to_string());
                    pkg.description = current_desc.trim().to_string();
                    packages.push(pkg);
                    current_desc.clear();
                    in_description = false;
                }
                continue;
            }

            if line.starts_with("Package: ") {
                // Starting a new stanza flushes the previous package (if any).
                if let Some(mut pkg) = current_pkg.take() {
                    pkg.section = section_mappings
                        .get(&pkg.name.to_ascii_lowercase())
                        .cloned()
                        .unwrap_or_else(|| "uncategorized".to_string());
                    pkg.description = current_desc.trim().to_string();
                    packages.push(pkg);
                    current_desc.clear();
                    in_description = false;
                }

                let name = line.strip_prefix("Package: ").unwrap_or("").to_string();
                let installed_version = installed.get(&name).cloned();
                let installed_flag = installed_version.is_some();

                current_pkg = Some(Package {
                    name,
                    version: String::new(),
                    installed: installed_flag,
                    installed_version,
                    size: None,
                    description: String::new(),
                    depends: Vec::new(),
                    priority: String::new(),
                    section: String::new(),
                    maintainer: String::new(),
                    architecture: String::new(),
                    homepage: None,
                });
            } else if let Some(pkg) = &mut current_pkg {
                // Parse known control fields; keep multiline Description text.
                if !has_leading_ws(line) {
                    in_description = false;
                }

                if let Some(v) = field_value(line, "Version") {
                    pkg.version = v;
                } else if let Some(v) = field_value(line, "Description") {
                    current_desc.push_str(&v);
                    in_description = true;
                } else if in_description && has_leading_ws(line) {
                    current_desc.push('\n');
                    let cont = line.trim_start();
                    if cont != "." {
                        current_desc.push_str(cont);
                    }
                } else if let Some(v) = field_value(line, "Depends") {
                    pkg.depends = v
                        .split(',')
                        .map(|s| {
                            s.split_whitespace()
                                .next()
                                .unwrap_or("")
                                .split('|')
                                .next()
                                .unwrap_or("")
                                .to_string()
                        })
                        .filter(|s| !s.is_empty())
                        .collect();
                } else if let Some(v) = field_value(line, "Size") {
                    pkg.size = v.parse().ok();
                } else if let Some(v) = field_value(line, "Installed-Size") {
                    if pkg.size.is_none() {
                        pkg.size = v.parse().ok();
                    }
                } else if let Some(v) = field_value(line, "Priority") {
                    pkg.priority = v;
                } else if let Some(v) = field_value(line, "Maintainer") {
                    pkg.maintainer = v;
                } else if let Some(v) = field_value(line, "Architecture") {
                    pkg.architecture = v;
                } else if let Some(v) = field_value(line, "Homepage") {
                    pkg.homepage = Some(v);
                }
            }
        }

        if let Some(mut pkg) = current_pkg {
            pkg.section = section_mappings
                .get(&pkg.name.to_ascii_lowercase())
                .cloned()
                .unwrap_or_else(|| "uncategorized".to_string());
            pkg.description = current_desc.trim().to_string();
            packages.push(pkg);
        }

        debug!("Found {} available packages", packages.len());
        Ok(packages)
    }
}
