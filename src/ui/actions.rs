//! UI action handlers:
//! menu dispatch, package marking, deferred apt operations, and search.
use super::*;

impl App {
    /// Executes the currently selected entry in the active popup menu.
    ///
    /// Dispatches menu labels to the corresponding app operation and closes
    /// the menu afterward.
    ///
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
                let added = self.mark_all_upgradable();
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
            (MenuKind::Search, "Find") => self.open_search_dialog_with_direction(true),
            (MenuKind::Search, "Find Backwards") => {
                self.open_search_dialog_with_direction(false)
            }
            (MenuKind::Search, "Find Again") => self.find_again(true),
            (MenuKind::Search, "Find Again Backwards") => self.find_again(false),
            (MenuKind::Options, "Preferences") => self.open_preferences_view(),
            (MenuKind::Options, "Revert options") => self.revert_options(),
            (MenuKind::Help, "About") => self.open_about_view(),
            (MenuKind::Help, "License") => self.open_license_view(),
            (MenuKind::Actions, "Play Minesweeper")
            | (MenuKind::Actions, "Become root")
            | (MenuKind::Package, "Forbid Version")
            | (MenuKind::Package, "Cycle Package Information")
            | (MenuKind::Search, "Limit Display")
            | (MenuKind::Search, "Un-Limit Display")
            | (MenuKind::Search, "Find Broken")
            | (MenuKind::Help, "Help")
            | (MenuKind::Help, "User's Manual")
            | (MenuKind::Help, "FAQ")
            | (MenuKind::Help, "News") => {
                self.status_message = Some(format!("'{}' is not implemented yet.", entry.label));
            }
            _ => {}
        }
        self.close_menu();
    }

    /// Returns the package name for the currently selected row.
    ///
    /// # Returns
    /// `Some(name)` when the selected row is a package row, otherwise `None`.
    ///
    pub(super) fn selected_package_name(&self) -> Option<&str> {
        match self.rows.get(self.selected_row).map(|r| &r.node) {
            Some(RowNode::Package(name)) => Some(name.as_str()),
            _ => None,
        }
    }

    /// Returns a clone of the selected tree node.
    ///
    /// # Returns
    /// `Some(node)` when a row is selected, otherwise `None`.
    ///
    pub(super) fn selected_node(&self) -> Option<RowNode> {
        self.rows.get(self.selected_row).map(|row| row.node.clone())
    }

    /// Marks all currently upgradable packages for installation/upgrade.
    ///
    /// # Returns
    /// Number of packages newly added to the pending-install set.
    ///
    pub(super) fn mark_all_upgradable(&mut self) -> usize {
        let mut added = 0usize;
        for name in &self.upgradable_names {
            self.pending_remove_names.remove(name);
            if self.pending_install_names.insert(name.clone()) {
                added += 1;
            }
        }
        added
    }

    /// Marks upgradable packages within the selected scope.
    ///
    /// Supported scopes in the Upgradable tree are:
    /// - group row (`Upgradable Packages`)
    /// - section row
    /// - archive row (`main`)
    ///
    /// # Arguments
    /// - `node`: The selected tree node that defines the scope.
    ///
    /// # Returns
    /// `Some((count, label))` when the node is an upgradable scope,
    /// otherwise `None`.
    ///
    pub(super) fn mark_upgradable_in_scope(&mut self, node: &RowNode) -> Option<(usize, String)> {
        let (names, scope_label): (Vec<String>, String) = match node {
            RowNode::Group(GroupKind::Upgradable) => {
                let mut names: Vec<String> = self.upgradable_names.iter().cloned().collect();
                names.sort();
                (names, "upgradable group".to_string())
            }
            RowNode::Section(GroupKind::Upgradable, section) => {
                let mut names: Vec<String> = self
                    .all_packages
                    .iter()
                    .filter(|p| self.upgradable_names.contains(&p.name) && p.section == *section)
                    .map(|p| p.name.clone())
                    .collect();
                names.sort();
                (names, section.clone())
            }
            RowNode::Archive(GroupKind::Upgradable, section, archive) => {
                let mut names: Vec<String> = self
                    .all_packages
                    .iter()
                    .filter(|p| self.upgradable_names.contains(&p.name) && p.section == *section)
                    .map(|p| p.name.clone())
                    .collect();
                names.sort();
                (names, format!("{section}/{archive}"))
            }
            _ => return None,
        };

        let mut added = 0usize;
        for name in names {
            self.pending_remove_names.remove(&name);
            if self.pending_install_names.insert(name) {
                added += 1;
            }
        }
        Some((added, scope_label))
    }

    /// Handles install mark action for the current selection.
    ///
    /// For upgradable group/section/archive rows, this marks all upgradable
    /// items under that scope. For package rows, it marks only that package.
    ///
    pub(super) fn mark_selected_package_install(&mut self) {
        if let Some(node) = self.selected_node() {
            if let Some((added, scope_label)) = self.mark_upgradable_in_scope(&node) {
                self.rebuild_rows();
                self.status_message = Some(format!(
                    "Marked {} upgradable packages in '{}'.",
                    added, scope_label
                ));
                return;
            }
        }

        let Some(name) = self.selected_package_name().map(ToString::to_string) else {
            self.status_message = Some("No package selected.".to_string());
            return;
        };
        self.pending_remove_names.remove(&name);
        self.pending_install_names.insert(name.clone());
        self.rebuild_rows();
        self.status_message = Some(format!("Marked '{}' for install.", name));
    }

    /// Marks the selected package for reinstall.
    ///
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

    /// Marks the selected package for removal.
    ///
    /// If the package is already pending install, this cancels that pending
    /// install instead of adding a remove mark.
    ///
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

    /// Clears pending install/remove marks for the selected package.
    ///
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

    /// Applies `apt-mark hold` to the selected package.
    ///
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

    /// Applies `apt-mark auto|manual` to the selected package.
    ///
    /// # Arguments
    /// - `auto`: `true` for `auto`, `false` for `manual`.
    ///
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

    /// Runs deferred long operations outside key/mouse handlers.
    ///
    /// # Arguments
    /// - `action`: Deferred operation to execute (`update` or `apply`).
    ///
    pub(super) fn run_deferred_action(&mut self, action: DeferredAction) {
        match action {
            DeferredAction::UpdatePackageList => self.start_update_action(),
            DeferredAction::ApplyPendingActions => self.start_apply_action(),
        }
    }

    /// Switches to the update output view and schedules `apt update`.
    ///
    pub(super) fn begin_update_list_view(&mut self) {
        if self.running_action.is_some() {
            self.status_message = Some("Another operation is already running.".to_string());
            return;
        }
        self.view_mode = ViewMode::UpdateList;
        self.update_scroll = 0;
        self.update_lines.clear();
        self.update_status = "Downloading...".to_string();
        self.status_message = Some("Updating package list...".to_string());
        self.deferred_action = Some(DeferredAction::UpdatePackageList);
    }

    /// Switches to the apply output view and schedules pending actions.
    ///
    pub(super) fn begin_apply_pending_view(&mut self) {
        if self.running_action.is_some() {
            self.status_message = Some("Another operation is already running.".to_string());
            return;
        }
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

    fn start_update_action(&mut self) {
        let queue = vec![CommandSpec {
            program: "apt".to_string(),
            args: vec!["update".to_string()],
        }];
        self.running_action = Some(RunningAction {
            kind: RunningActionKind::UpdatePackageList,
            queue,
            current: None,
            failed: false,
            started_at: Instant::now(),
        });
        if !self.start_next_running_command() {
            self.update_status = "Failed to start update.".to_string();
            self.status_message = Some("Failed to start package list update.".to_string());
        }
    }

    fn start_apply_action(&mut self) {
        let mut queue = Vec::new();
        let mut installs: Vec<String> = self.pending_install_names.iter().cloned().collect();
        installs.sort();
        if !installs.is_empty() {
            let mut args = vec!["install".to_string(), "-y".to_string()];
            args.extend(installs);
            queue.push(CommandSpec {
                program: "apt".to_string(),
                args,
            });
        }

        let mut removes: Vec<String> = self.pending_remove_names.iter().cloned().collect();
        removes.sort();
        if !removes.is_empty() {
            let mut args = vec!["remove".to_string(), "-y".to_string()];
            args.extend(removes);
            queue.push(CommandSpec {
                program: "apt".to_string(),
                args,
            });
        }

        if queue.is_empty() {
            self.update_status = "No pending actions.".to_string();
            self.update_lines = vec!["No pending actions to apply.".to_string()];
            self.status_message = Some("No pending actions.".to_string());
            return;
        }

        self.running_action = Some(RunningAction {
            kind: RunningActionKind::ApplyPendingActions,
            queue,
            current: None,
            failed: false,
            started_at: Instant::now(),
        });
        if !self.start_next_running_command() {
            self.update_status = "Failed to start apply.".to_string();
            self.status_message = Some("Failed to start pending apply.".to_string());
        }
    }

    fn spawn_streaming_command(spec: &CommandSpec) -> anyhow::Result<RunningCommand> {
        let mut child = Command::new(&spec.program)
            .args(&spec.args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let stdout = child.stdout.take();
        let stderr = child.stderr.take();
        let (tx, rx) = mpsc::channel::<String>();

        if let Some(out) = stdout {
            let tx_out = tx.clone();
            thread::spawn(move || {
                let reader = BufReader::new(out);
                for line in reader.lines().map_while(Result::ok) {
                    let _ = tx_out.send(line);
                }
            });
        }

        if let Some(err) = stderr {
            let tx_err = tx.clone();
            thread::spawn(move || {
                let reader = BufReader::new(err);
                for line in reader.lines().map_while(Result::ok) {
                    let _ = tx_err.send(line);
                }
            });
        }
        drop(tx);

        Ok(RunningCommand { child, rx })
    }

    fn push_update_line(&mut self, line: String) {
        for raw in line.replace('\r', "\n").lines() {
            let trimmed = raw.trim();
            if trimmed.is_empty() {
                continue;
            }
            self.update_lines.push(trimmed.to_string());
        }
        if self.update_lines.len() > 400 {
            let keep_from = self.update_lines.len() - 400;
            self.update_lines.drain(0..keep_from);
        }

        let viewport = self.list_area.height as usize;
        if viewport > 0 {
            self.update_scroll = self.update_lines.len().saturating_sub(viewport);
        }
    }

    fn start_next_running_command(&mut self) -> bool {
        let spec = {
            let Some(running) = self.running_action.as_mut() else {
                return false;
            };
            if running.queue.is_empty() {
                return false;
            }
            running.queue.remove(0)
        };

        let cmd_preview = format!("$ {} {}", spec.program, spec.args.join(" "));
        self.push_update_line(cmd_preview);

        match Self::spawn_streaming_command(&spec) {
            Ok(cmd) => {
                self.update_status = format!("Running: {} {}", spec.program, spec.args.join(" "));
                if let Some(running) = self.running_action.as_mut() {
                    running.current = Some(cmd);
                }
                true
            }
            Err(err) => {
                self.push_update_line(format!(
                    "Failed to start command '{}': {}",
                    spec.program, err
                ));
                if let Some(running) = self.running_action.as_mut() {
                    running.failed = true;
                    running.current = None;
                    running.queue.clear();
                }
                false
            }
        }
    }

    fn complete_running_action(&mut self) {
        let Some(running) = self.running_action.take() else {
            return;
        };
        let elapsed = running.started_at.elapsed().as_secs_f32();
        let mut had_error = running.failed;

        match running.kind {
            RunningActionKind::UpdatePackageList => {
                if running.failed {
                    self.update_status = "List update failed. Press Esc/Enter.".to_string();
                    self.status_message = Some("Failed to update package list.".to_string());
                } else if self.package_cache.refresh_after_update().is_ok() {
                    self.refresh_data();
                    let msg = format!("List update complete in {:.1}s. Press Esc/Enter.", elapsed);
                    self.update_status = msg.clone();
                    self.status_message = Some(msg);
                } else {
                    had_error = true;
                    self.update_status = "List update failed while rebuilding cache.".to_string();
                    self.status_message = Some("Failed to update package list.".to_string());
                }
            }
            RunningActionKind::ApplyPendingActions => {
                if running.failed {
                    self.update_status = "Apply failed. Press Esc/Enter.".to_string();
                    self.status_message = Some("Failed to apply some pending actions.".to_string());
                } else {
                    self.pending_install_names.clear();
                    self.pending_remove_names.clear();
                    if self.package_cache.refresh_after_update().is_err() {
                        had_error = true;
                    }
                    self.refresh_data();
                    let msg = format!("Apply complete in {:.1}s. Press Esc/Enter.", elapsed);
                    self.update_status = msg.clone();
                    self.status_message = Some(msg);
                }
            }
        }

        let should_pause = match self.options.pause_after_download {
            PauseAfterDownload::Never => false,
            PauseAfterDownload::OnlyIfError => had_error,
            PauseAfterDownload::Always => true,
        };

        if !should_pause {
            self.view_mode = ViewMode::Browser;
            self.update_scroll = 0;
            self.deferred_action = None;
        }
    }

    pub(super) fn tick_running_action(&mut self) {
        let mut emitted_lines: Vec<String> = Vec::new();
        let mut should_start_next = false;
        let mut should_complete = false;
        let mut status_poll_error: Option<String> = None;

        if let Some(running) = self.running_action.as_mut() {
            if let Some(current) = running.current.as_mut() {
                while let Ok(line) = current.rx.try_recv() {
                    emitted_lines.push(line);
                }

                match current.child.try_wait() {
                    Ok(Some(status)) => {
                        while let Ok(line) = current.rx.try_recv() {
                            emitted_lines.push(line);
                        }
                        if !status.success() {
                            running.failed = true;
                            running.queue.clear();
                        }
                        running.current = None;
                        should_start_next = true;
                    }
                    Ok(None) => {}
                    Err(err) => {
                        status_poll_error = Some(format!("Failed to poll command status: {}", err));
                        running.failed = true;
                        running.queue.clear();
                        running.current = None;
                        should_start_next = true;
                    }
                }
            } else {
                should_start_next = true;
            }
        }

        for line in emitted_lines {
            self.push_update_line(line);
        }
        if let Some(err) = status_poll_error {
            self.push_update_line(err);
        }

        if should_start_next {
            if !self.start_next_running_command() {
                should_complete = true;
            }
        }
        if should_complete {
            self.complete_running_action();
        }
    }

    /// Opens pending review view or reports when there is nothing pending.
    ///
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

    /// Routes key events for active overlays.
    ///
    /// Handles search dialog input and exit confirmation keys.
    ///
    pub(super) fn handle_overlay_key(&mut self, code: KeyCode) {
        match self.active_overlay {
            Some(OverlayKind::SearchDialog) => match code {
                KeyCode::Esc => self.active_overlay = None,
                KeyCode::Tab => {
                    self.search_dialog_focus = match self.search_dialog_focus {
                        SearchDialogFocus::Input => SearchDialogFocus::Ok,
                        SearchDialogFocus::Ok => SearchDialogFocus::Cancel,
                        SearchDialogFocus::Cancel => SearchDialogFocus::Input,
                    };
                }
                KeyCode::BackTab => {
                    self.search_dialog_focus = match self.search_dialog_focus {
                        SearchDialogFocus::Input => SearchDialogFocus::Cancel,
                        SearchDialogFocus::Ok => SearchDialogFocus::Input,
                        SearchDialogFocus::Cancel => SearchDialogFocus::Ok,
                    };
                }
                KeyCode::Left => {
                    self.search_dialog_focus = match self.search_dialog_focus {
                        SearchDialogFocus::Cancel => SearchDialogFocus::Ok,
                        SearchDialogFocus::Ok => SearchDialogFocus::Input,
                        SearchDialogFocus::Input => SearchDialogFocus::Input,
                    };
                }
                KeyCode::Right => {
                    self.search_dialog_focus = match self.search_dialog_focus {
                        SearchDialogFocus::Input => SearchDialogFocus::Ok,
                        SearchDialogFocus::Ok => SearchDialogFocus::Cancel,
                        SearchDialogFocus::Cancel => SearchDialogFocus::Cancel,
                    };
                }
                KeyCode::Down => {
                    self.search_dialog_focus = match self.search_dialog_focus {
                        SearchDialogFocus::Input => SearchDialogFocus::Ok,
                        other => other,
                    };
                }
                KeyCode::Up => {
                    self.search_dialog_focus = SearchDialogFocus::Input;
                }
                KeyCode::Enter => {
                    match self.search_dialog_focus {
                        SearchDialogFocus::Input | SearchDialogFocus::Ok => {
                            self.active_overlay = None;
                            self.execute_search(self.search_dialog_forward);
                        }
                        SearchDialogFocus::Cancel => {
                            self.active_overlay = None;
                        }
                    }
                }
                KeyCode::Backspace => {
                    if self.search_dialog_focus == SearchDialogFocus::Input {
                        self.search_input.pop();
                        if self.options.incremental_search {
                            self.execute_search(self.search_dialog_forward);
                        }
                    }
                }
                KeyCode::Char(c) => {
                    if self.search_dialog_focus != SearchDialogFocus::Input {
                        self.search_dialog_focus = SearchDialogFocus::Input;
                    }
                    self.search_input.push(c);
                    if self.options.incremental_search {
                        self.execute_search(self.search_dialog_forward);
                    }
                }
                _ => {}
            },
            Some(OverlayKind::ExitConfirm) => match code {
                KeyCode::Esc | KeyCode::Char('n') | KeyCode::Char('N') => {
                    self.active_overlay = None;
                }
                KeyCode::Enter | KeyCode::Char('y') | KeyCode::Char('Y') => {
                    if matches!(code, KeyCode::Enter) {
                        if self.exit_confirm_yes_selected {
                            self.should_quit = true;
                        } else {
                            self.active_overlay = None;
                        }
                    } else {
                        self.should_quit = true;
                    }
                }
                KeyCode::Left | KeyCode::Right | KeyCode::Tab => {
                    self.exit_confirm_yes_selected = !self.exit_confirm_yes_selected;
                }
                _ => {}
            },
            None => {}
        }
    }

    /// Opens search dialog and preloads the last query.
    ///
    pub(super) fn open_search_dialog(&mut self) {
        self.open_search_dialog_with_direction(true);
    }

    pub(super) fn open_search_dialog_with_direction(&mut self, forward: bool) {
        self.search_input = self.last_search_query.clone().unwrap_or_default();
        self.search_dialog_focus = SearchDialogFocus::Input;
        self.search_dialog_forward = forward;
        self.active_overlay = Some(OverlayKind::SearchDialog);
    }

    /// Runs a new search from current input.
    ///
    /// # Arguments
    /// - `forward`: Search direction.
    ///
    pub(super) fn execute_search(&mut self, forward: bool) {
        let query = self.search_input.trim().to_string();
        if query.is_empty() {
            self.status_message = Some("Search query is empty.".to_string());
            return;
        }
        self.last_search_query = Some(query.clone());
        self.find_again_internal(&query, forward);
    }

    /// Repeats the last search in the given direction.
    ///
    /// # Arguments
    /// - `forward`: Search direction.
    ///
    pub(super) fn find_again(&mut self, forward: bool) {
        let Some(query) = self.last_search_query.clone() else {
            self.status_message = Some("No previous search.".to_string());
            return;
        };
        self.find_again_internal(&query, forward);
    }

    /// Searches visible rows for a query and moves selection to next match.
    ///
    /// Expands all groups once if no match is found in the current collapsed
    /// tree, then retries.
    ///
    /// # Arguments
    /// - `query`: Case-insensitive query string.
    /// - `forward`: Search direction.
    ///
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

    /// Collects row indexes whose rendered text matches `query_lc`.
    ///
    /// # Arguments
    /// - `query_lc`: Lower-cased query string.
    ///
    /// # Returns
    /// Matching row indexes in display order.
    ///
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

    /// Expands all groups/sections/archives to maximize search visibility.
    ///
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

    /// Returns 1-based position of a matched row inside match list.
    ///
    /// # Returns
    /// Match ordinal; falls back to `1` when target is missing.
    ///
    pub(super) fn match_position(&self, matches: &[usize], target: usize) -> usize {
        matches
            .iter()
            .position(|idx| *idx == target)
            .map(|p| p + 1)
            .unwrap_or(1)
    }

    /// Updates list state selection to the current `selected_row`.
    ///
    pub(super) fn ensure_selected_row_visible(&mut self) {
        self.list_state.select(Some(self.selected_row));
    }
}
