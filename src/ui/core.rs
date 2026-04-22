//! Core app lifecycle and static UI definitions (menus, popup geometry).
use super::*;

impl App {
    /// Detect the current host name for the header line.
    ///
    /// Prefers the `hostname` command and falls back to standard
    /// environment variables.
    ///
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

    /// Split a multiline detail block into `(title, body)`.
    ///
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

    /// Return the static order of top menubar items.
    ///
    pub(super) fn top_menus() -> [MenuKind; 6] {
        [
            MenuKind::Actions,
            MenuKind::Undo,
            MenuKind::Package,
            MenuKind::Search,
            MenuKind::Options,
            MenuKind::Help,
        ]
    }

    /// Build popup menu entries for the given top-menu section.
    ///
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
                    label: "Information",
                    shortcut: "enter",
                    enabled: true,
                },
                MenuEntry {
                    kind: MenuEntryKind::Action,
                    label: "Cycle Package Information",
                    shortcut: "i",
                    enabled: false,
                },
                MenuEntry {
                    kind: MenuEntryKind::Action,
                    label: "Changelog",
                    shortcut: "C",
                    enabled: false,
                },
            ],
            MenuKind::Search => vec![
                MenuEntry {
                    kind: MenuEntryKind::Action,
                    label: "Find",
                    shortcut: "/",
                    enabled: true,
                },
                MenuEntry {
                    kind: MenuEntryKind::Action,
                    label: "Find Backwards",
                    shortcut: "\\",
                    enabled: true,
                },
                MenuEntry {
                    kind: MenuEntryKind::Action,
                    label: "Find Again",
                    shortcut: "n",
                    enabled: true,
                },
                MenuEntry {
                    kind: MenuEntryKind::Action,
                    label: "Find Again Backwards",
                    shortcut: "N",
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
                    label: "Limit Display",
                    shortcut: "l",
                    enabled: false,
                },
                MenuEntry {
                    kind: MenuEntryKind::Action,
                    label: "Un-Limit Display",
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
                    label: "Find Broken",
                    shortcut: "b",
                    enabled: false,
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

    /// Return the first selectable action index in a popup menu.
    ///
    pub(super) fn first_action_index(entries: &[MenuEntry]) -> usize {
        entries
            .iter()
            .position(|e| e.kind == MenuEntryKind::Action && e.enabled)
            .unwrap_or(0)
    }

    /// Compute popup rectangle anchored under its menubar label.
    ///
    pub(super) fn menu_popup_rect(kind: MenuKind) -> Rect {
        let entries = Self::menu_entries(kind);
        let content_width = entries
            .iter()
            .filter(|e| e.kind == MenuEntryKind::Action)
            .map(|e| {
                if e.shortcut.is_empty() {
                    e.label.len()
                } else {
                    e.label.len() + 1 + e.shortcut.len()
                }
            })
            .max()
            .unwrap_or(8);
        let width = (content_width + 3) as u16; // marker + borders
        let height = (entries.len() + 2) as u16;
        // Anchor directly under top-bar menu labels:
        // "Actions  Undo  Package ..."
        //   0        9     15
        let x = match kind {
            MenuKind::Actions => 0,
            MenuKind::Undo => 9,
            MenuKind::Package => 15,
            MenuKind::Search => 34,
            MenuKind::Options => 42,
            MenuKind::Help => 58,
        };
        Rect::new(x, 1, width.max(12), height.max(3))
    }

    /// Construct application state and load initial package data.
    ///
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
            exit_confirm_yes_selected: false,
            search_dialog_focus: SearchDialogFocus::Input,
            view_mode: ViewMode::Browser,
            pending_review_scroll: 0,
            update_lines: Vec::new(),
            update_scroll: 0,
            update_status: String::new(),
            running_action: None,
            search_input: String::new(),
            search_dialog_forward: true,
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

    /// Run the main UI event loop until the app exits.
    ///
    pub fn run(&mut self, terminal: &mut Terminal<impl ratatui::backend::Backend>) -> Result<()> {
        loop {
            self.tick_running_action();
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
