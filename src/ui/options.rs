use super::*;

impl App {
    pub(super) fn options_file_path() -> PathBuf {
        let base = dirs::config_dir()
            .or_else(dirs::home_dir)
            .unwrap_or_else(|| PathBuf::from("."));
        base.join("muxitude").join("options.json")
    }

    pub(super) fn load_options(path: &PathBuf) -> UiOptions {
        let Ok(raw) = std::fs::read_to_string(path) else {
            return UiOptions::default();
        };
        serde_json::from_str(&raw).unwrap_or_default()
    }

    pub(super) fn save_options(&mut self) {
        if let Some(parent) = self.options_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(data) = serde_json::to_string_pretty(&self.options) {
            let _ = std::fs::write(&self.options_path, data);
        }
    }

    pub(super) fn default_option_value(key: &'static str) -> String {
        let d = UiOptions::default();
        match key {
            "muxitude::UI::HelpBar" => d.help_bar.to_string(),
            "muxitude::UI::Menubar-Autohide" => d.menubar_autohide.to_string(),
            "muxitude::UI::Minibuf-Prompts" => d.minibuf_prompts.to_string(),
            "muxitude::UI::Incremental-Search" => d.incremental_search.to_string(),
            "muxitude::UI::Exit-On-Last-Close" => d.exit_on_last_close.to_string(),
            "muxitude::UI::Prompt-On-Exit" => d.prompt_on_exit.to_string(),
            "muxitude::UI::Pause-After-Download" => "OnlyIfError".to_string(),
            "muxitude::UI::Minibuf-Download-Bar" => d.status_line_download_bar.to_string(),
            "muxitude::UI::Description-Visible-By-Default" => {
                d.info_area_visible_by_default.to_string()
            }
            _ => "-".to_string(),
        }
    }

    pub(super) fn pause_value_str(value: PauseAfterDownload) -> &'static str {
        match value {
            PauseAfterDownload::Never => "No",
            PauseAfterDownload::OnlyIfError => "OnlyIfError",
            PauseAfterDownload::Always => "Yes",
        }
    }

    pub(super) fn current_option_value(&self, key: &'static str) -> String {
        match key {
            "muxitude::UI::HelpBar" => self.options.help_bar.to_string(),
            "muxitude::UI::Menubar-Autohide" => self.options.menubar_autohide.to_string(),
            "muxitude::UI::Minibuf-Prompts" => self.options.minibuf_prompts.to_string(),
            "muxitude::UI::Incremental-Search" => self.options.incremental_search.to_string(),
            "muxitude::UI::Exit-On-Last-Close" => self.options.exit_on_last_close.to_string(),
            "muxitude::UI::Prompt-On-Exit" => self.options.prompt_on_exit.to_string(),
            "muxitude::UI::Pause-After-Download" => {
                Self::pause_value_str(self.options.pause_after_download).to_string()
            }
            "muxitude::UI::Minibuf-Download-Bar" => {
                self.options.status_line_download_bar.to_string()
            }
            "muxitude::UI::Description-Visible-By-Default" => {
                self.options.info_area_visible_by_default.to_string()
            }
            _ => "-".to_string(),
        }
    }

    pub(super) fn build_preferences_rows(&self) -> Vec<PreferenceRow> {
        let bool_row = |label: String, key: &'static str, desc: &str| PreferenceRow {
            text: label,
            node: PreferenceNode::BoolOption { key },
            option_name: Some(key),
            default_value: Some(Self::default_option_value(key)),
            current_value: Some(self.current_option_value(key)),
            long_description: desc.to_string(),
        };

        let mut rows = vec![PreferenceRow {
            text: "--\\ UI options".to_string(),
            node: PreferenceNode::GroupHeader,
            option_name: None,
            default_value: None,
            current_value: None,
            long_description: "Change user-interface behavior.".to_string(),
        }];

        rows.push(bool_row(
            format!(
                "[{}] Display some available commands at the top of the screen",
                if self.options.help_bar { "X" } else { " " }
            ),
            "muxitude::UI::HelpBar",
            "If enabled, a brief summary of important commands appears beneath the menu bar.",
        ));
        rows.push(bool_row(
            format!(
                "[{}] Hide the menu bar when it is not being used",
                if self.options.menubar_autohide {
                    "X"
                } else {
                    " "
                }
            ),
            "muxitude::UI::Menubar-Autohide",
            "If enabled, the menu bar is hidden until activated.",
        ));
        rows.push(bool_row(
            format!(
                "[{}] Use a minibuffer-style prompt when possible",
                if self.options.minibuf_prompts {
                    "X"
                } else {
                    " "
                }
            ),
            "muxitude::UI::Minibuf-Prompts",
            "If enabled, prompts are displayed in the bottom info area instead of pop-up dialogs.",
        ));
        rows.push(bool_row(
            format!(
                "[{}] Show partial search results (incremental search)",
                if self.options.incremental_search {
                    "X"
                } else {
                    " "
                }
            ),
            "muxitude::UI::Incremental-Search",
            "If enabled, search updates while you type in the search dialog.",
        ));
        rows.push(bool_row(
            format!(
                "[{}] Closing the last view exits the program",
                if self.options.exit_on_last_close {
                    "X"
                } else {
                    " "
                }
            ),
            "muxitude::UI::Exit-On-Last-Close",
            "Compatibility option; currently there is a single main view.",
        ));
        rows.push(bool_row(
            format!(
                "[{}] Prompt for confirmation at exit",
                if self.options.prompt_on_exit {
                    "X"
                } else {
                    " "
                }
            ),
            "muxitude::UI::Prompt-On-Exit",
            "If enabled, quitting asks for confirmation.",
        ));
        rows.push(PreferenceRow {
            text: "--\\ Pause after downloading files".to_string(),
            node: PreferenceNode::PauseHeader,
            option_name: Some("muxitude::UI::Pause-After-Download"),
            default_value: Some(Self::default_option_value(
                "muxitude::UI::Pause-After-Download",
            )),
            current_value: Some(self.current_option_value("muxitude::UI::Pause-After-Download")),
            long_description: "Controls whether to pause before continuing after downloads."
                .to_string(),
        });
        for (value, label, desc) in [
            (
                PauseAfterDownload::Never,
                "Never",
                "Never wait after downloads.",
            ),
            (
                PauseAfterDownload::OnlyIfError,
                "When an error occurs",
                "Wait only when a download error occurred.",
            ),
            (
                PauseAfterDownload::Always,
                "Always",
                "Always wait for confirmation after downloads.",
            ),
        ] {
            let selected = self.options.pause_after_download == value;
            rows.push(PreferenceRow {
                text: format!("  ({}) {}", if selected { "*" } else { " " }, label),
                node: PreferenceNode::PauseChoice(value),
                option_name: Some("muxitude::UI::Pause-After-Download"),
                default_value: Some(Self::default_option_value(
                    "muxitude::UI::Pause-After-Download",
                )),
                current_value: Some(
                    self.current_option_value("muxitude::UI::Pause-After-Download"),
                ),
                long_description: desc.to_string(),
            });
        }
        rows.push(bool_row(
            format!(
                "[{}] Use a 'status-line' download indicator for all downloads",
                if self.options.status_line_download_bar {
                    "X"
                } else {
                    " "
                }
            ),
            "muxitude::UI::Minibuf-Download-Bar",
            "Compatibility option for aptitude-style download rendering.",
        ));
        rows.push(bool_row(
            format!(
                "[{}] Display the information area by default",
                if self.options.info_area_visible_by_default {
                    "X"
                } else {
                    " "
                }
            ),
            "muxitude::UI::Description-Visible-By-Default",
            "If enabled, the bottom information pane is shown by default.",
        ));
        rows
    }

    pub(super) fn refresh_preferences_rows(&mut self) {
        self.preferences_rows = self.build_preferences_rows();
        if self.preferences_rows.is_empty() {
            self.preferences_selected_row = 0;
        } else if self.preferences_selected_row >= self.preferences_rows.len() {
            self.preferences_selected_row = self.preferences_rows.len() - 1;
        }
    }

    pub(super) fn open_preferences_view(&mut self) {
        self.view_mode = ViewMode::Preferences;
        self.refresh_preferences_rows();
        self.preferences_selected_row =
            self.next_selectable_preference_index(self.preferences_rows.len().saturating_sub(1), 1);
        self.status_message = Some("Preferences".to_string());
    }

    pub(super) fn revert_options(&mut self) {
        self.options = UiOptions::default();
        self.info_area_visible = self.options.info_area_visible_by_default;
        self.save_options();
        self.refresh_preferences_rows();
        self.status_message = Some("Options reverted to defaults.".to_string());
    }

    pub(super) fn selected_preference_row(&self) -> Option<&PreferenceRow> {
        self.preferences_rows.get(self.preferences_selected_row)
    }

    pub(super) fn preference_row_selectable(row: &PreferenceRow) -> bool {
        !matches!(
            row.node,
            PreferenceNode::GroupHeader | PreferenceNode::PauseHeader
        )
    }

    pub(super) fn next_selectable_preference_index(&self, start: usize, delta: i32) -> usize {
        if self.preferences_rows.is_empty() {
            return 0;
        }
        let len = self.preferences_rows.len() as i32;
        let mut idx = start as i32;
        for _ in 0..len {
            idx = (idx + delta + len) % len;
            if let Some(row) = self.preferences_rows.get(idx as usize) {
                if Self::preference_row_selectable(row) {
                    return idx as usize;
                }
            }
        }
        start
    }

    pub(super) fn activate_selected_preference(&mut self) {
        let Some(row) = self.selected_preference_row().cloned() else {
            return;
        };
        match row.node {
            PreferenceNode::BoolOption { key } => {
                match key {
                    "muxitude::UI::HelpBar" => self.options.help_bar = !self.options.help_bar,
                    "muxitude::UI::Menubar-Autohide" => {
                        self.options.menubar_autohide = !self.options.menubar_autohide
                    }
                    "muxitude::UI::Minibuf-Prompts" => {
                        self.options.minibuf_prompts = !self.options.minibuf_prompts
                    }
                    "muxitude::UI::Incremental-Search" => {
                        self.options.incremental_search = !self.options.incremental_search
                    }
                    "muxitude::UI::Exit-On-Last-Close" => {
                        self.options.exit_on_last_close = !self.options.exit_on_last_close
                    }
                    "muxitude::UI::Prompt-On-Exit" => {
                        self.options.prompt_on_exit = !self.options.prompt_on_exit
                    }
                    "muxitude::UI::Minibuf-Download-Bar" => {
                        self.options.status_line_download_bar =
                            !self.options.status_line_download_bar
                    }
                    "muxitude::UI::Description-Visible-By-Default" => {
                        self.options.info_area_visible_by_default =
                            !self.options.info_area_visible_by_default;
                        self.info_area_visible = self.options.info_area_visible_by_default;
                    }
                    _ => {}
                }
                self.save_options();
                self.refresh_preferences_rows();
            }
            PreferenceNode::PauseChoice(choice) => {
                self.options.pause_after_download = choice;
                self.save_options();
                self.refresh_preferences_rows();
            }
            PreferenceNode::GroupHeader | PreferenceNode::PauseHeader => {}
        }
    }
}
