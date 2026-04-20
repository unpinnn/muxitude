use super::*;

impl App {
    // Prefer system hostname command for Termux/Linux parity.
    pub(super) fn detect_host_name() -> String {
        if let Ok(output) = Command::new("hostname").output() {
            if let Ok(name) = String::from_utf8(output.stdout) {
                let trimmed = name.trim();
                if !trimmed.is_empty() {
                    return trimmed.to_string();
                }
            }
        }
        std::env::var("HOSTNAME")
            .or_else(|_| std::env::var("COMPUTERNAME"))
            .unwrap_or_else(|_| "host".to_string())
    }

    pub(super) fn split_detail_text(detail: &str) -> (String, String) {
        let mut lines = detail.lines();
        let title = lines
            .next()
            .unwrap_or("No information available.")
            .trim()
            .to_string();
        let body = lines.collect::<Vec<_>>().join("\n").trim().to_string();
        (title, body)
    }

    pub(super) fn top_menus() -> [MenuKind; 5] {
        [
            MenuKind::Actions,
            MenuKind::Undo,
            MenuKind::Package,
            MenuKind::Options,
            MenuKind::Help,
        ]
    }

    pub(super) fn menu_entries(kind: MenuKind) -> Vec<MenuEntry> {
        match kind {
            MenuKind::Actions => vec![
                MenuEntry {
                    kind: MenuEntryKind::Action,
                    label: "Install/remove packages",
                    shortcut: "g",
                    enabled: true,
                },
                MenuEntry {
                    kind: MenuEntryKind::Action,
                    label: "Update package list",
                    shortcut: "u",
                    enabled: true,
                },
                MenuEntry {
                    kind: MenuEntryKind::Separator,
                    label: "",
                    shortcut: "",
                    enabled: false,
                },
                MenuEntry {
                    kind: MenuEntryKind::Action,
                    label: "Mark Upgradable",
                    shortcut: "U",
                    enabled: true,
                },
                MenuEntry {
                    kind: MenuEntryKind::Action,
                    label: "Forget new packages",
                    shortcut: "f",
                    enabled: true,
                },
                MenuEntry {
                    kind: MenuEntryKind::Action,
                    label: "Cancel pending actions",
                    shortcut: "",
                    enabled: true,
                },
                MenuEntry {
                    kind: MenuEntryKind::Action,
                    label: "Clean package cache",
                    shortcut: "",
                    enabled: true,
                },
                MenuEntry {
                    kind: MenuEntryKind::Action,
                    label: "Clean obsolete files",
                    shortcut: "",
                    enabled: true,
                },
                MenuEntry {
                    kind: MenuEntryKind::Separator,
                    label: "",
                    shortcut: "",
                    enabled: false,
                },
                MenuEntry {
                    kind: MenuEntryKind::Action,
                    label: "Play Minesweeper",
                    shortcut: "",
                    enabled: false,
                },
                MenuEntry {
                    kind: MenuEntryKind::Separator,
                    label: "",
                    shortcut: "",
                    enabled: false,
                },
                MenuEntry {
                    kind: MenuEntryKind::Action,
                    label: "Become root",
                    shortcut: "",
                    enabled: false,
                },
                MenuEntry {
                    kind: MenuEntryKind::Action,
                    label: "Quit",
                    shortcut: "Q",
                    enabled: true,
                },
            ],
            MenuKind::Undo => vec![MenuEntry {
                kind: MenuEntryKind::Action,
                label: "Undo",
                shortcut: "C-U",
                enabled: true,
            }],
            MenuKind::Package => vec![
                MenuEntry {
                    kind: MenuEntryKind::Action,
                    label: "Install",
                    shortcut: "+",
                    enabled: true,
                },
                MenuEntry {
                    kind: MenuEntryKind::Action,
                    label: "Reinstall",
                    shortcut: "L",
                    enabled: true,
                },
                MenuEntry {
                    kind: MenuEntryKind::Action,
                    label: "Remove",
                    shortcut: "-",
                    enabled: true,
                },
                MenuEntry {
                    kind: MenuEntryKind::Action,
                    label: "Purge",
                    shortcut: "_",
                    enabled: true,
                },
                MenuEntry {
                    kind: MenuEntryKind::Action,
                    label: "Keep",
                    shortcut: ":",
                    enabled: true,
                },
                MenuEntry {
                    kind: MenuEntryKind::Action,
                    label: "Hold",
                    shortcut: "=",
                    enabled: true,
                },
                MenuEntry {
                    kind: MenuEntryKind::Action,
                    label: "Mark Auto",
                    shortcut: "M",
                    enabled: true,
                },
                MenuEntry {
                    kind: MenuEntryKind::Action,
                    label: "Mark Manual",
                    shortcut: "m",
                    enabled: true,
                },
                MenuEntry {
                    kind: MenuEntryKind::Action,
                    label: "Forbid Version",
                    shortcut: "F",
                    enabled: true,
                },
                MenuEntry {
                    kind: MenuEntryKind::Separator,
                    label: "",
                    shortcut: "",
                    enabled: false,
                },
                MenuEntry {
                    kind: MenuEntryKind::Action,
                    label: "Information",
                    shortcut: "enter",
                    enabled: true,
                },
                MenuEntry {
                    kind: MenuEntryKind::Action,
                    label: "Cycle Package Information",
                    shortcut: "i",
                    enabled: true,
                },
                MenuEntry {
                    kind: MenuEntryKind::Action,
                    label: "Changelog",
                    shortcut: "C",
                    enabled: true,
                },
            ],
            MenuKind::Options => vec![
                MenuEntry {
                    kind: MenuEntryKind::Action,
                    label: "Preferences",
                    shortcut: "",
                    enabled: true,
                },
                MenuEntry {
                    kind: MenuEntryKind::Separator,
                    label: "",
                    shortcut: "",
                    enabled: false,
                },
                MenuEntry {
                    kind: MenuEntryKind::Action,
                    label: "Revert options",
                    shortcut: "",
                    enabled: true,
                },
            ],
            MenuKind::Help => vec![
                MenuEntry {
                    kind: MenuEntryKind::Action,
                    label: "About",
                    shortcut: "",
                    enabled: true,
                },
                MenuEntry {
                    kind: MenuEntryKind::Action,
                    label: "Help",
                    shortcut: "?",
                    enabled: false,
                },
                MenuEntry {
                    kind: MenuEntryKind::Action,
                    label: "User's Manual",
                    shortcut: "",
                    enabled: false,
                },
                MenuEntry {
                    kind: MenuEntryKind::Action,
                    label: "FAQ",
                    shortcut: "",
                    enabled: false,
                },
                MenuEntry {
                    kind: MenuEntryKind::Action,
                    label: "News",
                    shortcut: "",
                    enabled: false,
                },
                MenuEntry {
                    kind: MenuEntryKind::Action,
                    label: "License",
                    shortcut: "",
                    enabled: true,
                },
            ],
        }
    }

    pub(super) fn first_action_index(entries: &[MenuEntry]) -> usize {
        entries
            .iter()
            .position(|e| e.kind == MenuEntryKind::Action && e.enabled)
            .unwrap_or(0)
    }

    pub(super) fn menu_popup_rect(kind: MenuKind) -> Rect {
        let entries = Self::menu_entries(kind);
        let content_width = entries
            .iter()
            .filter(|e| e.kind == MenuEntryKind::Action)
            .map(|e| {
                let rhs = if e.shortcut.is_empty() {
                    0
                } else {
                    e.shortcut.len() + 1
                };
                e.label.len() + rhs + 1 // +1 for left marker/space
            })
            .max()
            .unwrap_or(8);
        let width = (content_width + 2) as u16; // borders
        let height = (entries.len() + 2) as u16;
        // Anchor directly under top-bar menu labels:
        // "Actions  Undo  Package ..."
        //   0        9     15
        let x = match kind {
            MenuKind::Actions => 0,
            MenuKind::Undo => 9,
            MenuKind::Package => 15,
            MenuKind::Options => 42,
            MenuKind::Help => 58,
        };
        Rect::new(x, 1, width.max(12), height.max(3))
    }

    // Construct app state and perform initial data load.
    pub fn new(package_cache: PackageCache) -> Self {
        let options_path = Self::options_file_path();
        let options = Self::load_options(&options_path);
        let mut app = Self {
            package_cache,
            should_quit: false,
            host_name: Self::detect_host_name(),
            app_version: env!("CARGO_PKG_VERSION"),
            groups: Vec::new(),
            rows: Vec::new(),
            selected_row: 0,
            all_packages: Vec::new(),
            upgradable_names: HashSet::new(),
            new_names: HashSet::new(),
            auto_installed_names: HashSet::new(),
            expanded_groups: HashSet::new(),
            expanded_sections: HashSet::new(),
            expanded_archives: HashSet::new(),
            list_state: ListState::default(),
            list_area: Rect::default(),
            last_click: None,
            active_menu: None,
            selected_menu_entry: 0,
            status_message: None,
            pending_install_names: HashSet::new(),
            pending_remove_names: HashSet::new(),
            deferred_action: None,
            active_overlay: None,
            view_mode: ViewMode::Browser,
            pending_review_scroll: 0,
            update_lines: Vec::new(),
            update_scroll: 0,
            update_status: String::new(),
            search_input: String::new(),
            last_search_query: None,
            options: options.clone(),
            options_path,
            info_area_visible: options.info_area_visible_by_default,
            preferences_rows: Vec::new(),
            preferences_selected_row: 0,
            help_page_title: String::new(),
            help_page_lines: Vec::new(),
            help_page_scroll: 0,
        };
        app.refresh_data();
        app.refresh_preferences_rows();
        app
    }

    // Main UI loop: draw, process deferred work, process input.
    pub fn run(&mut self, terminal: &mut Terminal<impl ratatui::backend::Backend>) -> Result<()> {
        loop {
            terminal.draw(|f| self.draw(f))?;

            if let Some(action) = self.deferred_action.take() {
                self.run_deferred_action(action);
                continue;
            }

            if event::poll(Duration::from_millis(100))? {
                match event::read()? {
                    Event::Key(key) if key.kind == KeyEventKind::Press => self.handle_key(key.code),
                    Event::Mouse(mouse) => self.handle_mouse(mouse),
                    _ => {}
                }
            }

            if self.should_quit {
                break;
            }
        }
        Ok(())
    }
}
