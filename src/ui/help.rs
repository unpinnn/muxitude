use super::*;

const LICENSE_TEXT: &str = include_str!("../../LICENSE");

impl App {
    pub(super) fn open_about_view(&mut self) {
        let lines = vec![
            format!("muxitude {}", self.app_version),
            String::new(),
            "A text-mode package browser/manager for Termux.".to_string(),
            "It follows aptitude-style keybindings where implemented.".to_string(),
            String::new(),
            "Keys and menu wording are intentionally close to aptitude".to_string(),
            "to make migration easier for existing aptitude users.".to_string(),
            String::new(),
            "License: GNU GPL-3.0".to_string(),
            "Source: https://github.com/unpinnn/muxitude".to_string(),
        ];
        self.open_help_page("About", lines);
    }

    pub(super) fn open_license_view(&mut self) {
        let lines = LICENSE_TEXT.lines().map(|line| line.to_string()).collect();
        self.open_help_page("License", lines);
    }

    pub(super) fn open_help_page(&mut self, title: &str, lines: Vec<String>) {
        self.view_mode = ViewMode::HelpPage;
        self.help_page_title = title.to_string();
        self.help_page_lines = lines;
        self.help_page_scroll = 0;
        self.status_message = Some(format!("{} (Esc to return)", title));
    }
}
