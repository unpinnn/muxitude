//! muxitude binary entrypoint.
//! Initializes cache layer and runs the terminal UI lifecycle.
mod pkgdb;
mod ui;

use anyhow::Result;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::env;
use std::io;
use std::path::PathBuf;

fn main() -> Result<()> {
    // Keep tracing initialized but silent in TUI mode.
    tracing_subscriber::fmt()
        .with_file(true)
        .with_line_number(true)
        .with_target(false)
        .with_writer(std::io::sink)
        .init();

    // Optional runtime override file for section mappings.
    let section_merge_path = parse_section_merge_arg()?;
    let pkg_cache = pkgdb::PackageCache::new_with_section_mappings_merge(section_merge_path)?;
    pkg_cache.refresh_if_needed()?;

    // Initialize terminal in alternate-screen/raw mode.
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = ui::App::new(pkg_cache);
    let res = app.run(&mut terminal);

    // Always restore terminal state before returning.
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    res
}

/// Parse supported CLI args.
///
/// Supported:
/// - `--section-mappings-merge <path>`
fn parse_section_merge_arg() -> Result<Option<PathBuf>> {
    // Minimal argument parser: only accepts --section-mappings-merge <path>.
    let mut args = env::args().skip(1);
    let mut merge_path: Option<PathBuf> = None;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--section-mappings-merge" => {
                let Some(path) = args.next() else {
                    anyhow::bail!("Missing value for --section-mappings-merge");
                };
                let pb = PathBuf::from(path);
                if !pb.exists() {
                    anyhow::bail!(
                        "--section-mappings-merge file does not exist: {}",
                        pb.display()
                    );
                }
                merge_path = Some(pb);
            }
            _ => anyhow::bail!("Unknown argument: {}", arg),
        }
    }

    Ok(merge_path)
}
