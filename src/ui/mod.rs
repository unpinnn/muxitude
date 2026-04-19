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
use std::collections::{HashMap, HashSet};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

#[derive(Clone, Copy, Hash, Eq, PartialEq)]
enum GroupKind {
    Upgradable,
    New,
    Installed,
    NotInstalled,
    ObsoleteLocal,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum MenuKind {
    Actions,
    Undo,
    Package,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum MenuEntryKind {
    Action,
    Separator,
}

#[derive(Clone)]
struct MenuEntry {
    kind: MenuEntryKind,
    label: &'static str,
    shortcut: &'static str,
    enabled: bool,
}

struct GroupItem {
    kind: GroupKind,
    name: String,
    count: usize,
    description: String,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum DeferredAction {
    UpdatePackageList,
    ApplyPendingActions,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum OverlayKind {
    SearchDialog,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ViewMode {
    Browser,
    PendingReview,
    UpdateList,
    ApplyPending,
}

#[derive(Clone)]
enum RowNode {
    Group(GroupKind),
    Section(GroupKind, String),
    Archive(GroupKind, String, String),
    Package(String),
}

#[derive(Clone)]
struct TreeRow {
    text: String,
    description: String,
    node: RowNode,
    style: Style,
}

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
    view_mode: ViewMode,
    pending_review_scroll: usize,
    update_lines: Vec<String>,
    update_scroll: usize,
    update_status: String,
    search_input: String,
    last_search_query: Option<String>,
}

mod actions;
mod core;
mod data;
mod input;
mod render;
