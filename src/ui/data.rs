use super::*;

impl App {
    // Pull fresh package sets from cache layer and rebuild visible tree.
    pub(super) fn refresh_data(&mut self) {
        self.all_packages = self.package_cache.get_all().unwrap_or_default();
        self.upgradable_names = self
            .package_cache
            .get_upgradable()
            .unwrap_or_default()
            .into_iter()
            .map(|p| p.name)
            .collect();
        self.new_names = self
            .package_cache
            .get_new_packages(14)
            .unwrap_or_default()
            .into_iter()
            .map(|p| p.name)
            .collect();
        self.auto_installed_names = Self::get_auto_installed_names();

        let installed_count = self.all_packages.iter().filter(|p| p.installed).count();
        let not_installed_count = self.all_packages.len().saturating_sub(installed_count);
        let obsolete_local_count = self
            .all_packages
            .iter()
            .filter(|p| {
                p.section.eq_ignore_ascii_case("local") || p.priority.eq_ignore_ascii_case("extra")
            })
            .count();

        let mut groups = Vec::new();

        if !self.upgradable_names.is_empty() {
            groups.push(GroupItem {
                kind: GroupKind::Upgradable,
                name: "Upgradable Packages".to_string(),
                count: self.upgradable_names.len(),
                description: format!(
                    "A newer version of these packages is available.\n\nThis group contains {} packages.",
                    self.upgradable_names.len()
                ),
            });
        }
        if !self.new_names.is_empty() {
            groups.push(GroupItem {
                kind: GroupKind::New,
                name: "New Packages".to_string(),
                count: self.new_names.len(),
                description: format!(
                    "Packages first seen in the last 14 days and not installed.\n\nThis group contains {} packages.",
                    self.new_names.len()
                ),
            });
        }
        groups.push(GroupItem {
            kind: GroupKind::Installed,
            name: "Installed Packages".to_string(),
            count: installed_count,
            description: format!(
                "These packages are currently installed on your computer.\n\nThis group contains {} packages.",
                installed_count
            ),
        });
        groups.push(GroupItem {
            kind: GroupKind::NotInstalled,
            name: "Not Installed Packages".to_string(),
            count: not_installed_count,
            description: format!(
                "These packages are available but not currently installed.\n\nThis group contains {} packages.",
                not_installed_count
            ),
        });
        if obsolete_local_count > 0 {
            groups.push(GroupItem {
                kind: GroupKind::ObsoleteLocal,
                name: "Obsolete and Locally Created Packages".to_string(),
                count: obsolete_local_count,
                description: format!(
                    "Packages that appear local/obsolete according to cached metadata.\n\nThis group contains {} packages.",
                    obsolete_local_count
                ),
            });
        }

        self.groups = groups;
        self.rebuild_rows();
    }

    pub(super) fn package_in_group(&self, p: &Package, kind: GroupKind) -> bool {
        match kind {
            GroupKind::Upgradable => self.upgradable_names.contains(&p.name),
            GroupKind::New => self.new_names.contains(&p.name),
            GroupKind::Installed => p.installed,
            GroupKind::NotInstalled => !p.installed,
            GroupKind::ObsoleteLocal => {
                p.section.eq_ignore_ascii_case("local") || p.priority.eq_ignore_ascii_case("extra")
            }
        }
    }

    pub(super) fn get_auto_installed_names() -> HashSet<String> {
        let output = Command::new("apt-mark").arg("showauto").output();
        let Ok(output) = output else {
            return HashSet::new();
        };
        let Ok(stdout) = String::from_utf8(output.stdout) else {
            return HashSet::new();
        };
        stdout
            .lines()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect()
    }

    // Build aptitude-style 3-level tree rows from current group/section state.
    pub(super) fn rebuild_rows(&mut self) {
        self.rows.clear();
        for g in &self.groups {
            let group_prefix = if self.expanded_groups.contains(&g.kind) {
                "--\\"
            } else {
                "---"
            };
            self.rows.push(TreeRow {
                text: format!("{group_prefix} {} ({})", g.name, g.count),
                description: g.description.clone(),
                node: RowNode::Group(g.kind),
                style: Style::default().fg(Color::White).bg(Color::Black),
            });

            if !self.expanded_groups.contains(&g.kind) {
                continue;
            }

            let mut section_counts: HashMap<String, usize> = HashMap::new();
            for p in self
                .all_packages
                .iter()
                .filter(|p| self.package_in_group(p, g.kind))
            {
                *section_counts.entry(p.section.clone()).or_insert(0) += 1;
            }
            let mut sections: Vec<String> = section_counts.keys().cloned().collect();
            sections.sort_by(|a, b| {
                let a_uncat = a.eq_ignore_ascii_case("uncategorized");
                let b_uncat = b.eq_ignore_ascii_case("uncategorized");
                match (a_uncat, b_uncat) {
                    (true, true) | (false, false) => a.cmp(b),
                    (true, false) => std::cmp::Ordering::Greater,
                    (false, true) => std::cmp::Ordering::Less,
                }
            });

            for section in sections {
                let count = *section_counts.get(&section).unwrap_or(&0);
                let section_is_main = section.eq_ignore_ascii_case("main");
                let section_prefix = if self.expanded_sections.contains(&(g.kind, section.clone()))
                {
                    "  --\\"
                } else {
                    "  ---"
                };
                self.rows.push(TreeRow {
                    text: format!("{section_prefix} {} ({})", section, count),
                    description: format!(
                        "Packages in the '{}' section.\n\nThis group contains {} packages.",
                        section, count
                    ),
                    node: RowNode::Section(g.kind, section.clone()),
                    style: Style::default().fg(Color::White).bg(Color::Black),
                });

                if !self.expanded_sections.contains(&(g.kind, section.clone())) {
                    continue;
                }

                if !section_is_main {
                    let archive = "main".to_string();
                    let archive_key = (g.kind, section.clone(), archive.clone());
                    let archive_expanded = self.expanded_archives.contains(&archive_key);
                    let archive_prefix = if archive_expanded {
                        "    --\\"
                    } else {
                        "    ---"
                    };
                    self.rows.push(TreeRow {
                        text: format!("{archive_prefix} {archive} ({count})"),
                        description: format!("The main archive for '{}' in this view.", section),
                        node: RowNode::Archive(g.kind, section.clone(), archive),
                        style: Style::default().fg(Color::White).bg(Color::Black),
                    });

                    if !archive_expanded {
                        continue;
                    }
                }

                let mut pkgs: Vec<&Package> = self
                    .all_packages
                    .iter()
                    .filter(|p| self.package_in_group(p, g.kind) && p.section == section)
                    .collect();
                pkgs.sort_by(|a, b| a.name.cmp(&b.name));

                for p in pkgs {
                    let action_flag = if self.pending_remove_names.contains(&p.name) {
                        '-'
                    } else if self.pending_install_names.contains(&p.name) {
                        '+'
                    } else {
                        ' '
                    };
                    let state_flag = if p.installed { "i" } else { "p" };
                    let auto_flag = if p.installed && self.auto_installed_names.contains(&p.name) {
                        "A"
                    } else {
                        " "
                    };
                    let installed_ver = p
                        .installed_version
                        .clone()
                        .unwrap_or_else(|| "-".to_string());
                    let candidate_ver = if p.version.is_empty() {
                        "-".to_string()
                    } else {
                        p.version.clone()
                    };
                    let mut detail = p.name.clone();
                    detail.push('\n');
                    detail.push_str(p.description.trim());
                    if let Some(homepage) = &p.homepage {
                        if !homepage.trim().is_empty() {
                            detail.push_str("\nHomepage: ");
                            detail.push_str(homepage.trim());
                        }
                    }

                    // Match aptitude-like package row spacing:
                    // "i A   <name> <installed> <candidate>"
                    let row_style = if self.pending_install_names.contains(&p.name) {
                        Style::default().fg(Color::Black).bg(Color::Green)
                    } else if self.pending_remove_names.contains(&p.name) {
                        Style::default().fg(Color::Black).bg(Color::Magenta)
                    } else if p.installed {
                        Style::default().fg(Color::White).bg(Color::Black)
                    } else {
                        Style::default().fg(Color::Gray).bg(Color::Black)
                    };
                    self.rows.push(TreeRow {
                        text: format!(
                            "{state_flag:1} {auto_flag:1} {action_flag:1} {:<30} {:<16} {}",
                            p.name, installed_ver, candidate_ver
                        ),
                        description: detail,
                        node: RowNode::Package(p.name.clone()),
                        style: row_style,
                    });
                }
            }
        }

        if self.rows.is_empty() {
            self.selected_row = 0;
            self.list_state.select(None);
        } else {
            if self.selected_row >= self.rows.len() {
                self.selected_row = self.rows.len() - 1;
            }
            self.list_state.select(Some(self.selected_row));
        }
    }
}
