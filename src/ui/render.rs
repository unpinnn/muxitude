//! Frame rendering: browser tree, overlays, menus, and info pane.
use super::*;
use ratatui::layout::Alignment;

impl App {
    /// Render the full frame: menubar, main pane, info pane, and overlays.
    ///
    pub(super) fn draw(&mut self, frame: &mut Frame) {
        let menubar_hidden = self.options.menubar_autohide
            && self.active_menu.is_none()
            && self.view_mode == ViewMode::Browser;
        let top_bar_height = if menubar_hidden { 0 } else { 1 };
        let header_height = 1;
        let info_height = if self.info_area_visible { 7 } else { 0 };

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(top_bar_height),
                Constraint::Length(header_height),
                Constraint::Min(8),
                Constraint::Length(info_height),
            ])
            .split(frame.size());

        let bar_style = Style::default().fg(Color::White).bg(Color::Blue);
        let normal_style = Style::default().fg(Color::White).bg(Color::Black);
        let selected_style = Style::default().fg(Color::Black).bg(Color::Gray);

        let menu_item_style = |active: bool| {
            if active {
                Style::default().fg(Color::Black).bg(Color::Gray)
            } else {
                bar_style
            }
        };
        let top_line = Line::from(vec![
            Span::styled(
                "Actions",
                menu_item_style(self.active_menu == Some(MenuKind::Actions)),
            ),
            Span::styled("  ", bar_style),
            Span::styled(
                "Undo",
                menu_item_style(self.active_menu == Some(MenuKind::Undo)),
            ),
            Span::styled("  ", bar_style),
            Span::styled(
                "Package",
                menu_item_style(self.active_menu == Some(MenuKind::Package)),
            ),
            Span::styled("  Resolver  ", bar_style),
            Span::styled(
                "Search",
                menu_item_style(self.active_menu == Some(MenuKind::Search)),
            ),
            Span::styled("  ", bar_style),
            Span::styled(
                "Options",
                menu_item_style(self.active_menu == Some(MenuKind::Options)),
            ),
            Span::styled("  Views  ", bar_style),
            Span::styled(
                "Help",
                menu_item_style(self.active_menu == Some(MenuKind::Help)),
            ),
        ]);
        if top_bar_height > 0 {
            frame.render_widget(Paragraph::new(top_line).style(bar_style), chunks[0]);
        }
        let second_line = if self.view_mode == ViewMode::UpdateList {
            "            Packages                        List Update".to_string()
        } else if self.view_mode == ViewMode::ApplyPending {
            "            Packages                   Install/Remove".to_string()
        } else if self.view_mode == ViewMode::Preferences {
            "            Packages                       Preferences".to_string()
        } else if self.view_mode == ViewMode::HelpPage {
            format!(
                "            Packages                           {}",
                self.help_page_title
            )
        } else if self.options.help_bar && self.view_mode == ViewMode::Browser {
            format!(
                "muxitude {} @ {}   [g apply  u update  / find  q quit]",
                self.app_version, self.host_name
            )
        } else {
            format!("muxitude {} @ {}", self.app_version, self.host_name)
        };
        frame.render_widget(Paragraph::new(second_line).style(bar_style), chunks[1]);
        self.list_area = chunks[2];

        if self.view_mode == ViewMode::Preferences {
            let items: Vec<ListItem> = self
                .preferences_rows
                .iter()
                .map(|r| {
                    let style = match r.node {
                        PreferenceNode::GroupHeader | PreferenceNode::PauseHeader => {
                            Style::default().fg(Color::White).bg(Color::Black)
                        }
                        _ => Style::default().fg(Color::White).bg(Color::Black),
                    };
                    ListItem::new(Line::from(vec![Span::raw(r.text.clone())])).style(style)
                })
                .collect();
            let list = List::new(items)
                .style(normal_style)
                .highlight_style(selected_style);
            if self.preferences_rows.is_empty() {
                self.list_state.select(None);
            } else {
                self.list_state.select(Some(self.preferences_selected_row));
            }
            frame.render_stateful_widget(list, chunks[2], &mut self.list_state);
        } else if self.view_mode == ViewMode::UpdateList || self.view_mode == ViewMode::ApplyPending
        {
            let lines: Vec<Line> = self
                .update_lines
                .iter()
                .map(|line| {
                    let style = if line.contains("[Hit]") {
                        Style::default().fg(Color::Black).bg(Color::Green)
                    } else if line.contains("[Downloaded]")
                        || line.contains("Get:")
                        || line.contains("Fetched")
                    {
                        Style::default().fg(Color::Blue).bg(Color::Yellow)
                    } else {
                        normal_style
                    };
                    Line::from(Span::styled(line.clone(), style))
                })
                .collect();
            frame.render_widget(
                Paragraph::new(lines)
                    .style(normal_style)
                    .scroll((self.update_scroll as u16, 0)),
                chunks[2],
            );
        } else if self.view_mode == ViewMode::PendingReview {
            let lines = self.build_pending_review_lines(chunks[2].width as usize);
            frame.render_widget(
                Paragraph::new(lines)
                    .style(normal_style)
                    .scroll((self.pending_review_scroll as u16, 0)),
                chunks[2],
            );
        } else if self.view_mode == ViewMode::HelpPage {
            let lines: Vec<Line> = self
                .help_page_lines
                .iter()
                .map(|line| Line::from(Span::raw(line.clone())))
                .collect();
            frame.render_widget(
                Paragraph::new(lines)
                    .style(normal_style)
                    .scroll((self.help_page_scroll as u16, 0)),
                chunks[2],
            );
        } else {
            let items: Vec<ListItem> = self
                .rows
                .iter()
                .map(|r| ListItem::new(Line::from(vec![Span::raw(r.text.clone())])).style(r.style))
                .collect();
            let list = List::new(items)
                .style(normal_style)
                .highlight_style(selected_style);
            if self.rows.is_empty() {
                self.list_state.select(None);
            } else {
                self.list_state.select(Some(self.selected_row));
            }
            frame.render_stateful_widget(list, chunks[2], &mut self.list_state);
        }
        self.draw_menu_popup(frame);
        self.draw_overlay(frame);

        let selected = if self.view_mode == ViewMode::PendingReview
            || self.view_mode == ViewMode::UpdateList
            || self.view_mode == ViewMode::ApplyPending
            || self.view_mode == ViewMode::Preferences
            || self.view_mode == ViewMode::HelpPage
        {
            None
        } else {
            self.rows.get(self.selected_row)
        };
        let desc =
            if self.view_mode == ViewMode::UpdateList || self.view_mode == ViewMode::ApplyPending {
                String::new()
            } else if self.view_mode == ViewMode::PendingReview {
                "Review pending actions.\n\nPress g or Enter to apply.\nPress Esc to return."
                    .to_string()
            } else {
                selected
                    .map(|r| r.description.clone())
                    .unwrap_or_else(|| "No row selected.".to_string())
            };
        let (detail_title, detail_body) = match selected.map(|r| &r.node) {
            Some(RowNode::Package(_)) => Self::split_detail_text(&desc),
            _ => (String::new(), desc),
        };

        if info_height > 0 {
            let detail_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(1), Constraint::Min(1)])
                .split(chunks[3]);
            let title_text = if self.view_mode == ViewMode::Preferences {
                if let Some(row) = self.selected_preference_row() {
                    row.option_name.unwrap_or(" ").to_string()
                } else {
                    " ".to_string()
                }
            } else if detail_title.trim().is_empty() {
                " ".to_string()
            } else {
                detail_title
            };
            frame.render_widget(
                Paragraph::new(title_text).style(bar_style),
                detail_chunks[0],
            );
            let mut body = if self.view_mode == ViewMode::Preferences {
                if let Some(row) = self.selected_preference_row() {
                    let def = row.default_value.clone().unwrap_or_else(|| "-".to_string());
                    let cur = row.current_value.clone().unwrap_or_else(|| "-".to_string());
                    format!(
                        "Default: {}\nValue: {}\n\n{}",
                        def, cur, row.long_description
                    )
                } else {
                    String::new()
                }
            } else {
                detail_body
            };
            if self.view_mode == ViewMode::UpdateList || self.view_mode == ViewMode::ApplyPending {
                body = String::new();
                body.push_str(&self.update_status);
            } else if self.view_mode == ViewMode::HelpPage {
                body = "Press Esc to return.".to_string();
            } else if self.view_mode != ViewMode::PendingReview
                && self.view_mode != ViewMode::Preferences
            {
                if let Some(status) = &self.status_message {
                    if !body.is_empty() {
                        body.push_str("\n\n");
                    }
                    body.push_str(status);
                }
            }
            frame.render_widget(Paragraph::new(body).style(normal_style), detail_chunks[1]);
        }
    }

    /// Draw active modal overlay (search or exit confirmation).
    ///
    pub(super) fn draw_overlay(&self, frame: &mut Frame) {
        match self.active_overlay {
            Some(OverlayKind::SearchDialog) => self.draw_search_dialog(frame),
            Some(OverlayKind::ExitConfirm) => self.draw_exit_confirm(frame),
            None => {}
        }
    }

    /// Draw the search dialog overlay.
    ///
    pub(super) fn draw_search_dialog(&self, frame: &mut Frame) {
        let area = frame.size();
        let width = area.width.saturating_sub(4).min(78).max(44);
        let height = 6u16.min(area.height.saturating_sub(2)).max(5);
        let x = area.x + area.width.saturating_sub(width) / 2;
        let y = area.y + area.height.saturating_sub(height) / 2;
        let popup = Rect::new(x, y, width, height);
        frame.render_widget(Clear, popup);
        let panel = Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::Black).bg(Color::Gray));
        frame.render_widget(panel, popup);

        let inner = Rect::new(
            popup.x.saturating_add(1),
            popup.y.saturating_add(1),
            popup.width.saturating_sub(2),
            popup.height.saturating_sub(2),
        );
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
            ])
            .split(inner);

        frame.render_widget(
            Paragraph::new("Search for:")
                .style(Style::default().fg(Color::Black).bg(Color::Gray)),
            chunks[0],
        );

        let mut input = self.search_input.clone();
        let input_width = chunks[1].width as usize;
        if input.len() < input_width {
            input.push_str(&" ".repeat(input_width - input.len()));
        }
        let input_style = if self.search_dialog_focus == SearchDialogFocus::Input {
            Style::default().fg(Color::Black).bg(Color::White)
        } else {
            Style::default().fg(Color::DarkGray).bg(Color::White)
        };
        frame.render_widget(
            Paragraph::new(input).style(input_style),
            chunks[1],
        );

        frame.render_widget(
            Paragraph::new("─".repeat(chunks[2].width as usize))
                .style(Style::default().fg(Color::Black).bg(Color::Gray)),
            chunks[2],
        );

        let ok_style = if self.search_dialog_focus == SearchDialogFocus::Ok {
            Style::default().fg(Color::White).bg(Color::Blue)
        } else {
            Style::default().fg(Color::Black).bg(Color::Gray)
        };
        let cancel_style = if self.search_dialog_focus == SearchDialogFocus::Cancel {
            Style::default().fg(Color::White).bg(Color::Blue)
        } else {
            Style::default().fg(Color::Black).bg(Color::Gray)
        };
        let button_line = Line::from(vec![
            Span::styled(
                "  [ ",
                Style::default().fg(Color::Black).bg(Color::Gray),
            ),
            Span::styled("Ok", ok_style),
            Span::styled(
                " ]    [ ",
                Style::default().fg(Color::Black).bg(Color::Gray),
            ),
            Span::styled("Cancel", cancel_style),
            Span::styled(" ]", Style::default().fg(Color::Black).bg(Color::Gray)),
        ]);
        frame.render_widget(
            Paragraph::new(button_line)
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::Black).bg(Color::Gray)),
            chunks[3],
        );
    }

    /// Draw exit confirmation overlay.
    ///
    pub(super) fn draw_exit_confirm(&self, frame: &mut Frame) {
        let area = frame.size();
        let prompt = "Really quit Muxitude?";
        let max_width = area.width.saturating_sub(4);
        let desired_width = (prompt.len() as u16).saturating_add(8);
        let width = desired_width.clamp(40, 56).min(max_width.max(1));
        let height = 5u16.min(area.height.saturating_sub(2)).max(4);
        let x = area.x + area.width.saturating_sub(width) / 2;
        let y = area.y + area.height.saturating_sub(height) / 2;
        let popup = Rect::new(x, y, width, height);
        frame.render_widget(Clear, popup);
        let panel = Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::Black).bg(Color::Gray));
        frame.render_widget(panel, popup);

        let inner = Rect::new(
            popup.x.saturating_add(1),
            popup.y.saturating_add(1),
            popup.width.saturating_sub(2),
            popup.height.saturating_sub(2),
        );
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Length(1), Constraint::Length(1)])
            .split(inner);

        frame.render_widget(
            Paragraph::new(prompt)
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::Black).bg(Color::Gray)),
            chunks[0],
        );
        frame.render_widget(
            Paragraph::new("─".repeat(chunks[1].width as usize))
                .style(Style::default().fg(Color::Black).bg(Color::Gray)),
            chunks[1],
        );

        let (yes_style, no_style) = if self.exit_confirm_yes_selected {
            (
                Style::default().fg(Color::White).bg(Color::Blue),
                Style::default().fg(Color::Black).bg(Color::Gray),
            )
        } else {
            (
                Style::default().fg(Color::Black).bg(Color::Gray),
                Style::default().fg(Color::White).bg(Color::Blue),
            )
        };
        let buttons = Line::from(vec![
            Span::styled("[ ", Style::default().fg(Color::Black).bg(Color::Gray)),
            Span::styled("Yes", yes_style),
            Span::styled(
                " ]   [ ",
                Style::default().fg(Color::Black).bg(Color::Gray),
            ),
            Span::styled("No", no_style),
            Span::styled(" ]", Style::default().fg(Color::Black).bg(Color::Gray)),
        ]);
        frame.render_widget(
            Paragraph::new(buttons)
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::Black).bg(Color::Gray)),
            chunks[2],
        );
    }

    /// Build pending-review lines grouped by upgrade/install/remove actions.
    ///
    pub(super) fn build_pending_review_lines(&self, row_width: usize) -> Vec<Line<'static>> {
        let mut lines: Vec<Line<'static>> = Vec::new();

        let package_by_name: HashMap<&str, &Package> = self
            .all_packages
            .iter()
            .map(|p| (p.name.as_str(), p))
            .collect();

        let mut upgrades: Vec<&Package> = self
            .pending_install_names
            .iter()
            .filter_map(|n| package_by_name.get(n.as_str()).copied())
            .filter(|p| p.installed)
            .collect();
        let mut installs: Vec<&Package> = self
            .pending_install_names
            .iter()
            .filter_map(|n| package_by_name.get(n.as_str()).copied())
            .filter(|p| !p.installed)
            .collect();
        let mut removes: Vec<&Package> = self
            .pending_remove_names
            .iter()
            .filter_map(|n| package_by_name.get(n.as_str()).copied())
            .collect();

        upgrades.sort_by(|a, b| a.name.cmp(&b.name));
        installs.sort_by(|a, b| a.name.cmp(&b.name));
        removes.sort_by(|a, b| a.name.cmp(&b.name));
        let upgrade_count = upgrades.len();
        let install_count = installs.len();
        let remove_count = removes.len();

        if !upgrades.is_empty() {
            lines.push(Line::from(Span::raw(format!(
                "--\\ Packages to be upgraded ({})",
                upgrade_count
            ))));
            for p in upgrades {
                lines.push(Self::build_colored_pending_line(
                    "iu",
                    &p.name,
                    Color::Cyan,
                    row_width,
                ));
            }
        }
        if !installs.is_empty() {
            lines.push(Line::from(Span::raw(format!(
                "--\\ Packages to be installed ({})",
                install_count
            ))));
            for p in installs {
                lines.push(Self::build_colored_pending_line(
                    "pi",
                    &p.name,
                    Color::Green,
                    row_width,
                ));
            }
        }
        if !removes.is_empty() {
            lines.push(Line::from(Span::raw(format!(
                "--\\ Packages to be removed ({})",
                remove_count
            ))));
            for p in removes {
                lines.push(Self::build_colored_pending_line(
                    "ip",
                    &p.name,
                    Color::Magenta,
                    row_width,
                ));
            }
        }
        if upgrade_count == 0 && install_count == 0 && remove_count == 0 {
            lines.push(Line::from(Span::raw("No pending actions.")));
        }

        lines
    }

    /// Build one full-width colored line used by pending-review rendering.
    ///
    fn build_colored_pending_line(
        state: &str,
        package_name: &str,
        bg: Color,
        row_width: usize,
    ) -> Line<'static> {
        let mut text = format!("{state}  {package_name}");
        if row_width > text.len() {
            text.push_str(&" ".repeat(row_width - text.len()));
        }
        Line::from(Span::styled(text, Style::default().fg(Color::Black).bg(bg)))
    }

    /// Draw the currently active popup menu.
    ///
    pub(super) fn draw_menu_popup(&self, frame: &mut Frame) {
        let Some(kind) = self.active_menu else {
            return;
        };
        let entries = Self::menu_entries(kind);
        let area = Self::menu_popup_rect(kind);
        frame.render_widget(Clear, area);
        let border = Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White).bg(Color::Blue));
        frame.render_widget(border, area);

        let inner = Rect::new(
            area.x.saturating_add(1),
            area.y.saturating_add(1),
            area.width.saturating_sub(2),
            area.height.saturating_sub(2),
        );
        let mut lines: Vec<Line> = Vec::new();
        let text_width = inner.width.saturating_sub(1) as usize; // reserve 1 column for marker/space
        for (idx, entry) in entries.iter().enumerate() {
            if entry.kind == MenuEntryKind::Separator {
                lines.push(Line::from(Span::styled(
                    "\u{2500}".repeat(inner.width as usize),
                    Style::default().fg(Color::White).bg(Color::Blue),
                )));
                continue;
            }
            let mut label = entry.label.to_string();
            let right = entry.shortcut;
            if right.is_empty() {
                if text_width > label.len() {
                    label.push_str(&" ".repeat(text_width - label.len()));
                }
            } else {
                let gap = text_width
                    .saturating_sub(label.len())
                    .saturating_sub(right.len())
                    .max(1);
                label.push_str(&" ".repeat(gap));
                label.push_str(right);
            }
            let is_selected = idx == self.selected_menu_entry;
            let style = if is_selected {
                if entry.enabled {
                    Style::default().fg(Color::Black).bg(Color::Gray)
                } else {
                    Style::default().fg(Color::DarkGray).bg(Color::Gray)
                }
            } else if entry.enabled {
                Style::default().fg(Color::White).bg(Color::Blue)
            } else {
                Style::default().fg(Color::DarkGray).bg(Color::Blue)
            };
            let mut spans = vec![Span::styled(label, style)];
            if is_selected {
                spans.insert(
                    0,
                    Span::styled("|", Style::default().fg(Color::Green).bg(Color::Gray)),
                );
            } else {
                spans.insert(
                    0,
                    Span::styled(" ", Style::default().fg(Color::White).bg(Color::Blue)),
                );
            }
            lines.push(Line::from(spans));
        }
        frame.render_widget(Paragraph::new(lines), inner);
    }
}
