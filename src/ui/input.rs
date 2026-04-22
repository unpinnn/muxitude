//! Keyboard and mouse input routing for all UI modes.
use super::*;

impl App {
    /// Open a popup menu and select its first enabled action row.
    ///
    pub(super) fn open_menu(&mut self, kind: MenuKind) {
        let entries = Self::menu_entries(kind);
        self.active_menu = Some(kind);
        self.selected_menu_entry = Self::first_action_index(&entries);
    }

    /// Close the currently open popup menu.
    ///
    pub(super) fn close_menu(&mut self) {
        self.active_menu = None;
    }

    /// Move popup menu selection up or down by one enabled action row.
    ///
    pub(super) fn navigate_menu_vertical(&mut self, delta: i32) {
        let Some(kind) = self.active_menu else {
            return;
        };
        let entries = Self::menu_entries(kind);
        if entries.is_empty() {
            return;
        }
        let len = entries.len() as i32;
        let mut idx = self.selected_menu_entry as i32;
        for _ in 0..len {
            idx = (idx + delta + len) % len;
            if entries[idx as usize].kind == MenuEntryKind::Action && entries[idx as usize].enabled
            {
                self.selected_menu_entry = idx as usize;
                return;
            }
        }
    }

    /// Switch active popup menu horizontally across the top menubar.
    ///
    pub(super) fn switch_menu_horizontal(&mut self, delta: i32) {
        let Some(current) = self.active_menu else {
            return;
        };
        let menus = Self::top_menus();
        let current_idx = menus.iter().position(|m| *m == current).unwrap_or(0) as i32;
        let next_idx = (current_idx + delta + menus.len() as i32) % menus.len() as i32;
        self.open_menu(menus[next_idx as usize]);
    }

    /// Execute single-character shortcut for the currently open menu.
    ///
    /// Returns `true` when a shortcut matched and was executed.
    ///
    pub(super) fn execute_menu_shortcut_key(&mut self, code: KeyCode) -> bool {
        let Some(kind) = self.active_menu else {
            return false;
        };
        let KeyCode::Char(ch) = code else {
            return false;
        };
        let entries = Self::menu_entries(kind);
        for (idx, entry) in entries.iter().enumerate() {
            if entry.kind != MenuEntryKind::Action || !entry.enabled {
                continue;
            }
            if entry.shortcut.len() == 1 {
                let k = entry.shortcut.chars().next().unwrap_or_default();
                if ch == k {
                    self.selected_menu_entry = idx;
                    self.execute_menu_action();
                    return true;
                }
            }
        }
        false
    }

    /// Route keyboard input according to current overlay/view/menu state.
    ///
    pub(super) fn handle_key(&mut self, code: KeyCode) {
        if self.active_overlay.is_some() {
            self.handle_overlay_key(code);
            return;
        }

        if self.view_mode == ViewMode::Preferences {
            match code {
                KeyCode::Esc => {
                    self.view_mode = ViewMode::Browser;
                    self.status_message = Some("Returned to package browser.".to_string());
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    self.preferences_selected_row =
                        self.next_selectable_preference_index(self.preferences_selected_row, -1);
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    self.preferences_selected_row =
                        self.next_selectable_preference_index(self.preferences_selected_row, 1);
                }
                KeyCode::Enter | KeyCode::Char(' ') => self.activate_selected_preference(),
                _ => {}
            }
            return;
        }

        if self.view_mode == ViewMode::HelpPage {
            let viewport = self.list_area.height as usize;
            let max_scroll = self.help_page_lines.len().saturating_sub(viewport);
            match code {
                KeyCode::Esc | KeyCode::Enter => {
                    self.view_mode = ViewMode::Browser;
                    self.help_page_scroll = 0;
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    self.help_page_scroll = self.help_page_scroll.saturating_sub(1);
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    self.help_page_scroll = (self.help_page_scroll + 1).min(max_scroll);
                }
                KeyCode::PageUp => {
                    self.help_page_scroll = self.help_page_scroll.saturating_sub(10);
                }
                KeyCode::PageDown => {
                    self.help_page_scroll = (self.help_page_scroll + 10).min(max_scroll);
                }
                KeyCode::Home => self.help_page_scroll = 0,
                KeyCode::End => self.help_page_scroll = max_scroll,
                _ => {}
            }
            return;
        }

        if self.view_mode == ViewMode::UpdateList || self.view_mode == ViewMode::ApplyPending {
            let viewport = self.list_area.height as usize;
            let max_scroll = self.update_lines.len().saturating_sub(viewport);
            match code {
                KeyCode::Esc | KeyCode::Enter => {
                    self.view_mode = ViewMode::Browser;
                    self.update_scroll = 0;
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    self.update_scroll = self.update_scroll.saturating_sub(1);
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    self.update_scroll = (self.update_scroll + 1).min(max_scroll);
                }
                KeyCode::PageUp => {
                    self.update_scroll = self.update_scroll.saturating_sub(10);
                }
                KeyCode::PageDown => {
                    self.update_scroll = (self.update_scroll + 10).min(max_scroll);
                }
                KeyCode::Home => self.update_scroll = 0,
                KeyCode::End => self.update_scroll = max_scroll,
                _ => {}
            }
            return;
        }

        if self.view_mode == ViewMode::PendingReview {
            let total_lines = self.build_pending_review_lines(self.list_area.width as usize).len();
            let viewport = self.list_area.height as usize;
            let max_scroll = total_lines.saturating_sub(viewport);
            match code {
                KeyCode::Esc => {
                    self.view_mode = ViewMode::Browser;
                    self.pending_review_scroll = 0;
                    self.status_message = Some("Returned to package browser.".to_string());
                }
                KeyCode::Enter | KeyCode::Char('g') => {
                    self.begin_apply_pending_view();
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    self.pending_review_scroll = self.pending_review_scroll.saturating_sub(1);
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    self.pending_review_scroll = (self.pending_review_scroll + 1).min(max_scroll);
                }
                KeyCode::PageUp => {
                    self.pending_review_scroll = self.pending_review_scroll.saturating_sub(10);
                }
                KeyCode::PageDown => {
                    self.pending_review_scroll = (self.pending_review_scroll + 10).min(max_scroll);
                }
                KeyCode::Home => self.pending_review_scroll = 0,
                KeyCode::End => self.pending_review_scroll = max_scroll,
                _ => {}
            }
            return;
        }

        if self.active_menu.is_some() {
            if self.execute_menu_shortcut_key(code) {
                return;
            }
            match code {
                KeyCode::Esc => self.close_menu(),
                KeyCode::Left => self.switch_menu_horizontal(-1),
                KeyCode::Right => self.switch_menu_horizontal(1),
                KeyCode::Up => self.navigate_menu_vertical(-1),
                KeyCode::Down => self.navigate_menu_vertical(1),
                KeyCode::Enter => self.execute_menu_action(),
                _ => {}
            }
            return;
        }

        match code {
            KeyCode::Char('q') => {
                if self.options.prompt_on_exit {
                    self.exit_confirm_yes_selected = false;
                    self.active_overlay = Some(OverlayKind::ExitConfirm);
                } else {
                    self.should_quit = true;
                }
            }
            KeyCode::Char('r') => self.refresh_data(),
            KeyCode::Char('/') => self.open_search_dialog(),
            KeyCode::Char('\\') => self.open_search_dialog_with_direction(false),
            KeyCode::Char('n') => self.find_again(true),
            KeyCode::Char('N') => self.find_again(false),
            KeyCode::Char('g') => self.open_pending_review_or_apply(),
            KeyCode::Char('u') => {
                self.begin_update_list_view();
            }
            KeyCode::Char('U') => {
                let added = self.mark_all_upgradable();
                self.rebuild_rows();
                self.status_message = Some(format!("Marked {} upgradable packages.", added));
            }
            KeyCode::Char('f') => {
                if self.package_cache.forget_new_packages().is_ok() {
                    self.refresh_data();
                    self.status_message = Some("Forgotten new package markers.".to_string());
                } else {
                    self.status_message = Some("Failed to forget new packages.".to_string());
                }
            }
            KeyCode::F(10) => self.open_menu(MenuKind::Actions),
            KeyCode::Char('+') => self.mark_selected_package_install(),
            KeyCode::Char('L') => self.mark_selected_package_reinstall(),
            KeyCode::Char('-') => self.mark_selected_package_remove(),
            KeyCode::Char('_') => self.mark_selected_package_remove(),
            KeyCode::Char(':') => self.clear_selected_package_marks(),
            KeyCode::Char('=') => self.hold_selected_package(),
            KeyCode::Char('M') => self.apt_mark_selected(true),
            KeyCode::Char('m') => self.apt_mark_selected(false),
            KeyCode::Up | KeyCode::Char('k') => {
                self.selected_row = self.selected_row.saturating_sub(1);
            }
            KeyCode::Down | KeyCode::Char('j') if self.selected_row + 1 < self.rows.len() => {
                self.selected_row += 1;
            }
            KeyCode::PageUp => {
                self.selected_row = self.selected_row.saturating_sub(10);
            }
            KeyCode::PageDown if !self.rows.is_empty() => {
                self.selected_row = (self.selected_row + 10).min(self.rows.len() - 1);
            }
            KeyCode::PageDown => {}
            KeyCode::Enter => self.toggle_selected_node(),
            KeyCode::Left => self.open_menu(MenuKind::Actions),
            KeyCode::Right => self.open_menu(MenuKind::Undo),
            KeyCode::Char('a') => self.open_menu(MenuKind::Actions),
            KeyCode::Char('z') => self.open_menu(MenuKind::Undo),
            KeyCode::Char('p') => self.open_menu(MenuKind::Package),
            KeyCode::Char('s') => self.open_menu(MenuKind::Search),
            _ => {}
        }
    }

    /// Route mouse input for menus, list selection, and scrolling.
    ///
    pub(super) fn handle_mouse(&mut self, mouse: MouseEvent) {
        if self.active_overlay.is_some() {
            if let MouseEventKind::Down(MouseButton::Left) = mouse.kind {
                self.active_overlay = None;
            }
            return;
        }
        if self.view_mode == ViewMode::UpdateList || self.view_mode == ViewMode::ApplyPending {
            let viewport = self.list_area.height as usize;
            let max_scroll = self.update_lines.len().saturating_sub(viewport);
            match mouse.kind {
                MouseEventKind::ScrollUp => {
                    self.update_scroll = self.update_scroll.saturating_sub(3);
                }
                MouseEventKind::ScrollDown => {
                    self.update_scroll = (self.update_scroll + 3).min(max_scroll);
                }
                _ => {}
            }
            return;
        }
        if self.view_mode == ViewMode::PendingReview {
            let total_lines = self.build_pending_review_lines(self.list_area.width as usize).len();
            let viewport = self.list_area.height as usize;
            let max_scroll = total_lines.saturating_sub(viewport);
            match mouse.kind {
                MouseEventKind::ScrollUp => {
                    self.pending_review_scroll = self.pending_review_scroll.saturating_sub(3);
                }
                MouseEventKind::ScrollDown => {
                    self.pending_review_scroll = (self.pending_review_scroll + 3).min(max_scroll);
                }
                _ => {}
            }
            return;
        }
        if self.view_mode == ViewMode::HelpPage {
            let viewport = self.list_area.height as usize;
            let max_scroll = self.help_page_lines.len().saturating_sub(viewport);
            match mouse.kind {
                MouseEventKind::ScrollUp => {
                    self.help_page_scroll = self.help_page_scroll.saturating_sub(3);
                }
                MouseEventKind::ScrollDown => {
                    self.help_page_scroll = (self.help_page_scroll + 3).min(max_scroll);
                }
                _ => {}
            }
            return;
        }

        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                // Top menu bar hitboxes (approximate aptitude positions)
                if mouse.row == 0 {
                    let x = mouse.column;
                    if (0..=7).contains(&x) {
                        self.open_menu(MenuKind::Actions);
                        return;
                    }
                    if (9..=12).contains(&x) {
                        self.open_menu(MenuKind::Undo);
                        return;
                    }
                    if (15..=21).contains(&x) {
                        self.open_menu(MenuKind::Package);
                        return;
                    }
                    if (34..=39).contains(&x) {
                        self.open_menu(MenuKind::Search);
                        return;
                    }
                    if (42..=48).contains(&x) {
                        self.open_menu(MenuKind::Options);
                        return;
                    }
                    if (58..=61).contains(&x) {
                        self.open_menu(MenuKind::Help);
                        return;
                    }
                    self.close_menu();
                }

                if let Some(kind) = self.active_menu {
                    let popup = Self::menu_popup_rect(kind);
                    if mouse.column >= popup.x
                        && mouse.column < popup.x + popup.width
                        && mouse.row >= popup.y
                        && mouse.row < popup.y + popup.height
                    {
                        let rel_y = mouse.row.saturating_sub(popup.y + 1) as usize;
                        let entries = Self::menu_entries(kind);
                        if rel_y < entries.len()
                            && entries[rel_y].kind == MenuEntryKind::Action
                            && entries[rel_y].enabled
                        {
                            self.selected_menu_entry = rel_y;
                            self.execute_menu_action();
                        }
                        return;
                    } else {
                        self.close_menu();
                    }
                }

                if !self.mouse_in_list(mouse.column, mouse.row) || self.rows.is_empty() {
                    return;
                }
                let idx = self.row_from_mouse(mouse.row);
                if idx >= self.rows.len() {
                    return;
                }
                self.selected_row = idx;

                let now = Instant::now();
                if let Some((last_idx, when)) = self.last_click {
                    if last_idx == idx && now.duration_since(when) <= Duration::from_millis(350) {
                        self.toggle_selected_node();
                        self.last_click = None;
                        return;
                    }
                }
                self.last_click = Some((idx, now));
            }
            MouseEventKind::ScrollUp => {
                self.selected_row = self.selected_row.saturating_sub(3);
            }
            MouseEventKind::ScrollDown if !self.rows.is_empty() => {
                self.selected_row = (self.selected_row + 3).min(self.rows.len() - 1);
            }
            MouseEventKind::ScrollDown => {}
            _ => {}
        }
    }

    /// Return whether a terminal coordinate is inside the list viewport.
    ///
    pub(super) fn mouse_in_list(&self, x: u16, y: u16) -> bool {
        x >= self.list_area.x
            && x < self.list_area.x.saturating_add(self.list_area.width)
            && y >= self.list_area.y
            && y < self.list_area.y.saturating_add(self.list_area.height)
    }

    /// Convert mouse `y` coordinate to row index in the backing row vector.
    ///
    pub(super) fn row_from_mouse(&self, y: u16) -> usize {
        let rel = y.saturating_sub(self.list_area.y) as usize;
        rel + self.list_state.offset()
    }

    /// Toggle expand/collapse for selected tree node when supported.
    ///
    pub(super) fn toggle_selected_node(&mut self) {
        let Some(row) = self.rows.get(self.selected_row).cloned() else {
            return;
        };
        match row.node {
            RowNode::Group(kind) => {
                if !self.expanded_groups.insert(kind) {
                    self.expanded_groups.remove(&kind);
                }
                self.rebuild_rows();
            }
            RowNode::Section(kind, section) => {
                let key = (kind, section);
                if !self.expanded_sections.insert(key.clone()) {
                    self.expanded_sections.remove(&key);
                }
                self.rebuild_rows();
            }
            RowNode::Archive(kind, section, archive) => {
                let key = (kind, section, archive);
                if !self.expanded_archives.insert(key.clone()) {
                    self.expanded_archives.remove(&key);
                }
                self.rebuild_rows();
            }
            RowNode::Package(_) => {}
        }
    }
}
