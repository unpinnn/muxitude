use super::*;

impl App {
    // Execute currently highlighted popup-menu action.
    pub(super) fn execute_menu_action(&mut self) {
        let Some(kind) = self.active_menu else {
            return;
        };
        let entries = Self::menu_entries(kind);
        let Some(entry) = entries.get(self.selected_menu_entry) else {
            return;
        };
        if entry.kind != MenuEntryKind::Action || !entry.enabled {
            return;
        }

        match (kind, entry.label) {
            (MenuKind::Actions, "Install/remove packages") => self.begin_apply_pending_view(),
            (MenuKind::Actions, "Update package list") => {
                self.begin_update_list_view();
            }
            (MenuKind::Actions, "Mark Upgradable") => {
                let mut added = 0usize;
                for name in &self.upgradable_names {
                    self.pending_remove_names.remove(name);
                    if self.pending_install_names.insert(name.clone()) {
                        added += 1;
                    }
                }
                self.rebuild_rows();
                self.status_message = Some(format!("Marked {} upgradable packages.", added));
            }
            (MenuKind::Actions, "Forget new packages") => {
                if self.package_cache.forget_new_packages().is_ok() {
                    self.refresh_data();
                    self.status_message = Some("Forgotten new package markers.".to_string());
                } else {
                    self.status_message = Some("Failed to forget new packages.".to_string());
                }
            }
            (MenuKind::Actions, "Cancel pending actions") => {
                let total = self.pending_install_names.len() + self.pending_remove_names.len();
                self.pending_install_names.clear();
                self.pending_remove_names.clear();
                self.rebuild_rows();
                self.status_message = Some(format!("Canceled {} pending actions.", total));
            }
            (MenuKind::Actions, "Clean package cache") => {
                let ok = Command::new("apt")
                    .arg("clean")
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .status()
                    .map(|s| s.success())
                    .unwrap_or(false);
                self.status_message = Some(if ok {
                    "Package cache cleaned.".to_string()
                } else {
                    "Failed to clean package cache.".to_string()
                });
            }
            (MenuKind::Actions, "Clean obsolete files") => {
                let ok = Command::new("apt")
                    .args(["autoremove", "-y"])
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .status()
                    .map(|s| s.success())
                    .unwrap_or(false);
                self.status_message = Some(if ok {
                    "Obsolete packages removed.".to_string()
                } else {
                    "Failed to clean obsolete files.".to_string()
                });
            }
            (MenuKind::Actions, "Quit") => self.should_quit = true,
            (MenuKind::Undo, "Undo") => {
                self.status_message = Some("Nothing to undo yet.".to_string());
            }
            (MenuKind::Package, "Install") | (MenuKind::Package, "Reinstall") => {
                if entry.label == "Reinstall" {
                    self.mark_selected_package_reinstall();
                } else {
                    self.mark_selected_package_install();
                }
            }
            (MenuKind::Package, "Remove") | (MenuKind::Package, "Purge") => {
                self.mark_selected_package_remove();
            }
            (MenuKind::Package, "Hold") => self.hold_selected_package(),
            (MenuKind::Package, "Keep") => self.clear_selected_package_marks(),
            (MenuKind::Package, "Mark Auto") => self.apt_mark_selected(true),
            (MenuKind::Package, "Mark Manual") => self.apt_mark_selected(false),
            (MenuKind::Package, "Information") => {
                self.status_message = Some("Package information shown below.".to_string());
            }
            (MenuKind::Package, "Changelog") => {
                self.status_message = Some("Changelog action is not implemented yet.".to_string());
            }
            (MenuKind::Actions, "Play Minesweeper")
            | (MenuKind::Actions, "Become root")
            | (MenuKind::Package, "Forbid Version")
            | (MenuKind::Package, "Cycle Package Information") => {
                self.status_message = Some(format!("'{}' is not implemented yet.", entry.label));
            }
            _ => {}
        }
        self.close_menu();
    }

    // Package-level mark helpers (non-destructive until apply phase).
    pub(super) fn selected_package_name(&self) -> Option<&str> {
        match self.rows.get(self.selected_row).map(|r| &r.node) {
            Some(RowNode::Package(name)) => Some(name.as_str()),
            _ => None,
        }
    }

    pub(super) fn mark_selected_package_install(&mut self) {
        let Some(name) = self.selected_package_name().map(ToString::to_string) else {
            self.status_message = Some("No package selected.".to_string());
            return;
        };
        self.pending_remove_names.remove(&name);
        self.pending_install_names.insert(name.clone());
        self.rebuild_rows();
        self.status_message = Some(format!("Marked '{}' for install.", name));
    }

    pub(super) fn mark_selected_package_reinstall(&mut self) {
        let Some(name) = self.selected_package_name().map(ToString::to_string) else {
            self.status_message = Some("No package selected.".to_string());
            return;
        };
        self.pending_remove_names.remove(&name);
        self.pending_install_names.insert(name.clone());
        self.rebuild_rows();
        self.status_message = Some(format!("Marked '{}' for reinstall.", name));
    }

    pub(super) fn mark_selected_package_remove(&mut self) {
        let Some(name) = self.selected_package_name().map(ToString::to_string) else {
            self.status_message = Some("No package selected.".to_string());
            return;
        };
        if self.pending_install_names.remove(&name) {
            self.rebuild_rows();
            self.status_message = Some(format!("Canceled pending install for '{}'.", name));
            return;
        }
        self.pending_remove_names.insert(name.clone());
        self.rebuild_rows();
        self.status_message = Some(format!("Marked '{}' for removal.", name));
    }

    pub(super) fn clear_selected_package_marks(&mut self) {
        let Some(name) = self.selected_package_name().map(ToString::to_string) else {
            self.status_message = Some("No package selected.".to_string());
            return;
        };
        let changed =
            self.pending_install_names.remove(&name) || self.pending_remove_names.remove(&name);
        if changed {
            self.rebuild_rows();
        }
        self.status_message = Some(if changed {
            format!("Cleared pending action for '{}'.", name)
        } else {
            format!("'{}' had no pending action.", name)
        });
    }

    pub(super) fn hold_selected_package(&mut self) {
        let Some(name) = self.selected_package_name().map(ToString::to_string) else {
            self.status_message = Some("No package selected.".to_string());
            return;
        };
        let ok = Command::new("apt-mark")
            .args(["hold", &name])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false);
        self.status_message = Some(if ok {
            format!("Marked '{}' on hold.", name)
        } else {
            format!("Failed to hold '{}'.", name)
        });
    }

    // Run apt-mark auto/manual for selected package.
    pub(super) fn apt_mark_selected(&mut self, auto: bool) {
        let Some(name) = self.selected_package_name().map(ToString::to_string) else {
            self.status_message = Some("No package selected.".to_string());
            return;
        };
        let mode = if auto { "auto" } else { "manual" };
        let ok = Command::new("apt-mark")
            .args([mode, &name])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false);
        self.status_message = Some(if ok {
            self.auto_installed_names = Self::get_auto_installed_names();
            format!("Marked '{}' as {}.", name, mode)
        } else {
            format!("Failed to mark '{}' as {}.", name, mode)
        });
    }

    // Deferred jobs run outside input handlers to keep redraw cycle stable.
    pub(super) fn run_deferred_action(&mut self, action: DeferredAction) {
        match action {
            DeferredAction::UpdatePackageList => {
                let started = Instant::now();
                let update_output = Command::new("apt").arg("update").output();

                match update_output {
                    Ok(output) => {
                        self.update_lines =
                            Self::parse_update_output(&output.stdout, &output.stderr);
                    }
                    Err(err) => {
                        self.update_lines = vec![format!("Failed to run apt update: {err}")];
                        self.update_status = "Update failed.".to_string();
                        self.status_message = Some("Failed to update package list.".to_string());
                        return;
                    }
                }

                if self.package_cache.refresh_after_update().is_ok() {
                    self.refresh_data();
                    let msg = format!(
                        "List update complete in {:.1}s. Press Esc/Enter.",
                        started.elapsed().as_secs_f32()
                    );
                    self.update_status = msg.clone();
                    self.status_message = Some(msg);
                } else {
                    self.update_status = "List update failed while rebuilding cache.".to_string();
                    self.status_message = Some("Failed to update package list.".to_string());
                }
            }
            DeferredAction::ApplyPendingActions => {
                let started = Instant::now();
                let installs: Vec<String> = self.pending_install_names.iter().cloned().collect();
                let removes: Vec<String> = self.pending_remove_names.iter().cloned().collect();
                if installs.is_empty() && removes.is_empty() {
                    self.update_lines = vec!["No pending actions to apply.".to_string()];
                    self.update_status = "No pending actions.".to_string();
                    return;
                }

                let mut all_lines: Vec<String> = Vec::new();
                let install_ok = if installs.is_empty() {
                    true
                } else {
                    match Command::new("apt")
                        .arg("install")
                        .arg("-y")
                        .args(&installs)
                        .output()
                    {
                        Ok(out) => {
                            all_lines.extend(Self::parse_update_output(&out.stdout, &out.stderr));
                            out.status.success()
                        }
                        Err(err) => {
                            all_lines.push(format!("Failed to run apt install: {err}"));
                            false
                        }
                    }
                };
                let remove_ok = if removes.is_empty() {
                    true
                } else {
                    match Command::new("apt")
                        .arg("remove")
                        .arg("-y")
                        .args(&removes)
                        .output()
                    {
                        Ok(out) => {
                            all_lines.extend(Self::parse_update_output(&out.stdout, &out.stderr));
                            out.status.success()
                        }
                        Err(err) => {
                            all_lines.push(format!("Failed to run apt remove: {err}"));
                            false
                        }
                    }
                };
                self.update_lines = if all_lines.is_empty() {
                    vec!["No output.".to_string()]
                } else {
                    all_lines
                };

                if install_ok && remove_ok {
                    self.pending_install_names.clear();
                    self.pending_remove_names.clear();
                    let _ = self.package_cache.refresh_after_update();
                    self.refresh_data();
                    let msg = format!(
                        "Apply complete in {:.1}s. Press Esc/Enter.",
                        started.elapsed().as_secs_f32()
                    );
                    self.update_status = msg.clone();
                    self.status_message = Some(msg);
                } else {
                    self.update_status = "Apply failed. Press Esc/Enter.".to_string();
                    self.status_message = Some("Failed to apply some pending actions.".to_string());
                }
            }
        }
    }

    // Enter aptitude-like update output screen and schedule apt update.
    pub(super) fn begin_update_list_view(&mut self) {
        self.view_mode = ViewMode::UpdateList;
        self.update_scroll = 0;
        self.update_lines.clear();
        self.update_status = "Downloading...".to_string();
        self.status_message = Some("Updating package list...".to_string());
        self.deferred_action = Some(DeferredAction::UpdatePackageList);
    }

    // Enter pending apply output screen and schedule install/remove execution.
    pub(super) fn begin_apply_pending_view(&mut self) {
        let has_pending =
            !self.pending_install_names.is_empty() || !self.pending_remove_names.is_empty();
        if !has_pending {
            self.status_message = Some("No pending actions to apply.".to_string());
            return;
        }
        self.view_mode = ViewMode::ApplyPending;
        self.update_scroll = 0;
        self.update_lines.clear();
        self.update_status = "Applying pending actions...".to_string();
        self.status_message = Some("Applying pending actions...".to_string());
        self.deferred_action = Some(DeferredAction::ApplyPendingActions);
    }

    pub(super) fn parse_update_output(stdout: &[u8], stderr: &[u8]) -> Vec<String> {
        let mut lines = Vec::new();
        let mut push_lines = |text: &str| {
            for raw in text.replace('\r', "\n").lines() {
                let line = raw.trim();
                if !line.is_empty() {
                    lines.push(line.to_string());
                }
            }
        };

        if let Ok(s) = String::from_utf8(stdout.to_vec()) {
            push_lines(&s);
        }
        if let Ok(s) = String::from_utf8(stderr.to_vec()) {
            push_lines(&s);
        }

        if lines.is_empty() {
            lines.push("No output from apt update.".to_string());
        }
        if lines.len() > 400 {
            lines = lines.split_off(lines.len() - 400);
        }
        lines
    }

    pub(super) fn open_pending_review_or_apply(&mut self) {
        let has_pending =
            !self.pending_install_names.is_empty() || !self.pending_remove_names.is_empty();
        if !has_pending {
            self.status_message = Some("No pending actions to review.".to_string());
            return;
        }
        self.view_mode = ViewMode::PendingReview;
        self.pending_review_scroll = 0;
        self.status_message = Some("Review pending actions. Press g again to apply.".to_string());
    }

    // Overlay key handler currently used by search dialog only.
    pub(super) fn handle_overlay_key(&mut self, code: KeyCode) {
        match self.active_overlay {
            Some(OverlayKind::SearchDialog) => match code {
                KeyCode::Esc => self.active_overlay = None,
                KeyCode::Enter => {
                    self.active_overlay = None;
                    self.execute_search(true);
                }
                KeyCode::Backspace => {
                    self.search_input.pop();
                }
                KeyCode::Char(c) => {
                    self.search_input.push(c);
                }
                _ => {}
            },
            None => {}
        }
    }

    // Search utilities: / to open, n/N to iterate matches.
    pub(super) fn open_search_dialog(&mut self) {
        self.search_input = self.last_search_query.clone().unwrap_or_default();
        self.active_overlay = Some(OverlayKind::SearchDialog);
    }

    pub(super) fn execute_search(&mut self, forward: bool) {
        let query = self.search_input.trim().to_string();
        if query.is_empty() {
            self.status_message = Some("Search query is empty.".to_string());
            return;
        }
        self.last_search_query = Some(query.clone());
        self.find_again_internal(&query, forward);
    }

    pub(super) fn find_again(&mut self, forward: bool) {
        let Some(query) = self.last_search_query.clone() else {
            self.status_message = Some("No previous search.".to_string());
            return;
        };
        self.find_again_internal(&query, forward);
    }

    pub(super) fn find_again_internal(&mut self, query: &str, forward: bool) {
        if self.rows.is_empty() {
            self.status_message = Some("No rows to search.".to_string());
            return;
        }
        let query_lc = query.to_ascii_lowercase();
        let mut matches = self.collect_row_matches(&query_lc);
        if matches.is_empty() {
            // If current tree is collapsed, search should still find packages.
            self.expand_all_for_search();
            self.rebuild_rows();
            matches = self.collect_row_matches(&query_lc);
            if matches.is_empty() {
                self.status_message = Some(format!("No matches for '{}'.", query));
                return;
            }
        }

        let next_idx = if forward {
            matches
                .iter()
                .copied()
                .find(|idx| *idx > self.selected_row)
                .unwrap_or(matches[0])
        } else {
            matches
                .iter()
                .rev()
                .copied()
                .find(|idx| *idx < self.selected_row)
                .unwrap_or(*matches.last().unwrap_or(&self.selected_row))
        };

        self.selected_row = next_idx;
        self.ensure_selected_row_visible();
        self.status_message = Some(format!(
            "Found '{}' ({}/{})",
            query,
            self.match_position(&matches, next_idx),
            matches.len()
        ));
    }

    pub(super) fn collect_row_matches(&self, query_lc: &str) -> Vec<usize> {
        self.rows
            .iter()
            .enumerate()
            .filter_map(|(idx, row)| {
                if row.text.to_ascii_lowercase().contains(query_lc) {
                    Some(idx)
                } else {
                    None
                }
            })
            .collect()
    }

    pub(super) fn expand_all_for_search(&mut self) {
        for g in &self.groups {
            self.expanded_groups.insert(g.kind);
            let mut sections: HashSet<String> = HashSet::new();
            for p in self
                .all_packages
                .iter()
                .filter(|p| self.package_in_group(p, g.kind))
            {
                sections.insert(p.section.clone());
            }
            for section in sections {
                self.expanded_sections.insert((g.kind, section.clone()));
                if !section.eq_ignore_ascii_case("main") {
                    self.expanded_archives
                        .insert((g.kind, section, "main".to_string()));
                }
            }
        }
    }

    pub(super) fn match_position(&self, matches: &[usize], target: usize) -> usize {
        matches
            .iter()
            .position(|idx| *idx == target)
            .map(|p| p + 1)
            .unwrap_or(1)
    }

    pub(super) fn ensure_selected_row_visible(&mut self) {
        self.list_state.select(Some(self.selected_row));
    }
}
