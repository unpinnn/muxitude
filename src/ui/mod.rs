//! Terminal UI state model and submodules.
//!
//! This module owns:
//! - application state (`App`)
//! - menu/row/view enums used by rendering and input handling
//! - shared imports for split UI implementation files
//! UI module split by responsibility.
//!
//! - `core`: app bootstrap and shared helpers
//! - `input`: keyboard/mouse handling
//! - `data`: tree/view-model rebuilding from package cache
//! - `actions`: side-effecting operations (apt, search, updates)
//! - `render`: ratatui drawing
use crate::pkgdb::{Package, PackageCache};
use anyhow::Result;
use crossterm::event::{
    self, Event, KeyCode, KeyEventKind, MouseButton, MouseEvent, MouseEventKind,
};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph};
use ratatui::{Frame, Terminal};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::{Duration, Instant};

#[derive(Clone, Copy, Hash, Eq, PartialEq)]
/// Top-level package groups shown in the browser tree.
enum GroupKind {
    Upgradable,
    New,
    Installed,
    NotInstalled,
    ObsoleteLocal,
}

#[derive(Clone, Copy, PartialEq, Eq)]
/// Root menubar entries.
enum MenuKind {
    Actions,
    Undo,
    Package,
    Search,
    Options,
    Help,
}

#[derive(Clone, Copy, PartialEq, Eq)]
/// Popup menu row type.
enum MenuEntryKind {
    Action,
    Separator,
}

#[derive(Clone)]
/// One popup menu row.
struct MenuEntry {
    kind: MenuEntryKind,
    label: &'static str,
    shortcut: &'static str,
    enabled: bool,
}

/// Rendered top-level group metadata.
struct GroupItem {
    kind: GroupKind,
    name: String,
    count: usize,
    description: String,
}

#[derive(Clone, Copy, PartialEq, Eq)]
/// Long-running action to run outside input handlers.
enum DeferredAction {
    UpdatePackageList,
    ApplyPendingActions,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum RunningActionKind {
    UpdatePackageList,
    ApplyPendingActions,
}

struct CommandSpec {
    program: String,
    args: Vec<String>,
}

struct RunningCommand {
    child: Child,
    rx: Receiver<String>,
}

struct RunningAction {
    kind: RunningActionKind,
    queue: Vec<CommandSpec>,
    current: Option<RunningCommand>,
    failed: bool,
    started_at: Instant,
}

#[derive(Clone, Copy, PartialEq, Eq)]
/// Modal overlays drawn above the current view.
enum OverlayKind {
    SearchDialog,
    ExitConfirm,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum SearchDialogFocus {
    Input,
    Ok,
    Cancel,
}

#[derive(Clone, Copy, PartialEq, Eq)]
/// Active main view.
enum ViewMode {
    Browser,
    PendingReview,
    UpdateList,
    ApplyPending,
    Preferences,
    HelpPage,
}

#[derive(Clone)]
/// Semantic node backing each visible tree row.
enum RowNode {
    Group(GroupKind),
    Section(GroupKind, String),
    Archive(GroupKind, String, String),
    Package(String),
}

#[derive(Clone)]
/// Render-ready row with display text and semantic metadata.
struct TreeRow {
    text: String,
    description: String,
    node: RowNode,
    style: Style,
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
/// Preference values for download pause behavior.
enum PauseAfterDownload {
    Never,
    OnlyIfError,
    Always,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
/// Persisted UI preference values.
struct UiOptions {
    help_bar: bool,
    menubar_autohide: bool,
    incremental_search: bool,
    exit_on_last_close: bool,
    prompt_on_exit: bool,
    pause_after_download: PauseAfterDownload,
    info_area_visible_by_default: bool,
}

impl Default for UiOptions {
    fn default() -> Self {
        Self {
            help_bar: true,
            menubar_autohide: false,
            incremental_search: true,
            exit_on_last_close: true,
            prompt_on_exit: true,
            pause_after_download: PauseAfterDownload::OnlyIfError,
            info_area_visible_by_default: true,
        }
    }
}

#[derive(Clone)]
/// Logical node type for the preferences tree.
enum PreferenceNode {
    GroupHeader,
    BoolOption { key: &'static str },
    PauseHeader,
    PauseChoice(PauseAfterDownload),
}

#[derive(Clone)]
/// Render-ready preferences row.
struct PreferenceRow {
    text: String,
    node: PreferenceNode,
    option_name: Option<&'static str>,
    default_value: Option<String>,
    current_value: Option<String>,
    long_description: String,
}

/// Main application controller for the muxitude terminal UI.
pub struct App {
    package_cache: PackageCache,
    should_quit: bool,
    host_name: String,
    app_version: &'static str,
    groups: Vec<GroupItem>,
    rows: Vec<TreeRow>,
    selected_row: usize,
    all_packages: Vec<Package>,
    upgradable_names: HashSet<String>,
    new_names: HashSet<String>,
    auto_installed_names: HashSet<String>,
    expanded_groups: HashSet<GroupKind>,
    expanded_sections: HashSet<(GroupKind, String)>,
    expanded_archives: HashSet<(GroupKind, String, String)>,
    list_state: ListState,
    list_area: Rect,
    last_click: Option<(usize, Instant)>,
    active_menu: Option<MenuKind>,
    selected_menu_entry: usize,
    status_message: Option<String>,
    pending_install_names: HashSet<String>,
    pending_remove_names: HashSet<String>,
    deferred_action: Option<DeferredAction>,
    active_overlay: Option<OverlayKind>,
    exit_confirm_yes_selected: bool,
    search_dialog_focus: SearchDialogFocus,
    view_mode: ViewMode,
    pending_review_scroll: usize,
    update_lines: Vec<String>,
    update_scroll: usize,
    update_status: String,
    running_action: Option<RunningAction>,
    search_input: String,
    search_dialog_forward: bool,
    last_search_query: Option<String>,
    options: UiOptions,
    options_path: PathBuf,
    info_area_visible: bool,
    preferences_rows: Vec<PreferenceRow>,
    preferences_selected_row: usize,
    help_page_title: String,
    help_page_lines: Vec<String>,
    help_page_scroll: usize,
}

mod actions;
mod core;
mod data;
mod help;
mod input;
mod options;
mod render;
