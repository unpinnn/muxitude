use super::*;

impl App {
    // Top-level renderer: menu bar, main pane, status/detail pane, overlays.
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
            Span::styled("  Resolver  Search  ", bar_style),
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
            let lines = self.build_pending_review_lines();
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
            if self.active_overlay == Some(OverlayKind::SearchDialog)
                && self.options.minibuf_prompts
            {
                body = format!(
                    "Search for:\n{}\n\nPress Enter to search, Esc to cancel.",
                    self.search_input
                );
            }
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

    // Draw transient overlays (search dialog, etc.).
    pub(super) fn draw_overlay(&self, frame: &mut Frame) {
        match self.active_overlay {
            Some(OverlayKind::SearchDialog) => self.draw_search_dialog(frame),
            Some(OverlayKind::ExitConfirm) => self.draw_exit_confirm(frame),
            None => {}
        }
    }

    pub(super) fn draw_search_dialog(&self, frame: &mut Frame) {
        if self.options.minibuf_prompts && self.info_area_visible {
            return;
        }
        let area = frame.size();
        let width = area.width.min(92);
        let height = 6u16;
        let x = area.x + area.width.saturating_sub(width) / 2;
        let y = area.y + area.height.saturating_sub(height) / 2;
        let popup = Rect::new(x, y, width, height);
        frame.render_widget(Clear, popup);
        frame.render_widget(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::White).bg(Color::Blue)),
            popup,
        );

        let prompt = "Search for:";
        let inner = Rect::new(
            popup.x.saturating_add(2),
            popup.y.saturating_add(1),
            popup.width.saturating_sub(4),
            popup.height.saturating_sub(2),
        );
        let lines = vec![
            Line::from(Span::styled(
                prompt,
                Style::default().fg(Color::White).bg(Color::Blue),
            )),
            Line::from(Span::styled(
                self.search_input.clone(),
                Style::default().fg(Color::Black).bg(Color::Gray),
            )),
            Line::from(Span::styled(
                "[ Enter: Search ]    [ Esc: Cancel ]",
                Style::default().fg(Color::White).bg(Color::Blue),
            )),
        ];
        frame.render_widget(
            Paragraph::new(lines).style(Style::default().bg(Color::Blue)),
            inner,
        );
    }

    pub(super) fn draw_exit_confirm(&self, frame: &mut Frame) {
        let area = frame.size();
        let width = 42u16.min(area.width);
        let height = 5u16.min(area.height);
        let x = area.x + area.width.saturating_sub(width) / 2;
        let y = area.y + area.height.saturating_sub(height) / 2;
        let popup = Rect::new(x, y, width, height);
        frame.render_widget(Clear, popup);
        frame.render_widget(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::White).bg(Color::Blue)),
            popup,
        );
        frame.render_widget(
            Paragraph::new("Quit muxitude?\n[ Enter / y ] Yes    [ Esc / n ] No")
                .style(Style::default().fg(Color::White).bg(Color::Blue)),
            Rect::new(
                popup.x.saturating_add(1),
                popup.y.saturating_add(1),
                popup.width.saturating_sub(2),
                popup.height.saturating_sub(2),
            ),
        );
    }

    // Build review screen rows for pending install/remove operations.
    pub(super) fn build_pending_review_lines(&self) -> Vec<Line<'static>> {
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
                lines.push(Line::from(Span::styled(
                    format!("iu  {}", p.name),
                    Style::default().fg(Color::Black).bg(Color::Cyan),
                )));
            }
        }
        if !installs.is_empty() {
            lines.push(Line::from(Span::raw(format!(
                "--\\ Packages to be installed ({})",
                install_count
            ))));
            for p in installs {
                lines.push(Line::from(Span::styled(
                    format!("pi  {}", p.name),
                    Style::default().fg(Color::Black).bg(Color::Green),
                )));
            }
        }
        if !removes.is_empty() {
            lines.push(Line::from(Span::raw(format!(
                "--\\ Packages to be removed ({})",
                remove_count
            ))));
            for p in removes {
                lines.push(Line::from(Span::styled(
                    format!("ip  {}", p.name),
                    Style::default().fg(Color::Black).bg(Color::Magenta),
                )));
            }
        }
        if upgrade_count == 0 && install_count == 0 && remove_count == 0 {
            lines.push(Line::from(Span::raw("No pending actions.")));
        }

        lines
    }

    // Draw the currently active top menu popup.
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
                    "-".repeat(inner.width as usize),
                    Style::default().fg(Color::White).bg(Color::Blue),
                )));
                continue;
            }
            let mut label = entry.label.to_string();
            let right = entry.shortcut;
            let fill = if right.is_empty() {
                0
            } else {
                text_width
                    .saturating_sub(label.len())
                    .saturating_sub(right.len())
                    .saturating_sub(1)
            };
            if fill > 0 {
                label.push_str(&" ".repeat(fill));
                label.push(' ');
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
