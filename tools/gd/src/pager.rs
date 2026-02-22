use std::io::{self, Write};
use std::time::Duration;

use crossterm::event::{self, Event, KeyEventKind};

use tui::pager::{
    ALT_SCREEN_OFF, ALT_SCREEN_ON, CLEAR_LINE, CURSOR_HIDE, CURSOR_SHOW, copy_to_clipboard,
    get_term_size, move_to,
};
use tui::search::{
    find_matches, find_nearest_match, highlight_search, max_scroll, word_boundary_left,
    word_boundary_right,
};

use crate::git::diff::{DiffFile, FileStatus, LineKind};
use crate::render::{self, LineInfo, RenderOutput};
use crate::style;

use tui::pager::Key;

/// Returns true when GD_DEBUG=1. Debug output is disabled by default.
fn gd_debug_enabled() -> bool {
    std::env::var_os("GD_DEBUG").map_or(false, |v| v == "1")
}

fn debug_escape(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

/// Writes a structured debug line to stderr only when GD_DEBUG=1. No-op otherwise.
/// Does not mutate state.
fn debug_trace(location: &str, message: &str, data: &str) {
    if !gd_debug_enabled() {
        return;
    }
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_millis() as u64);
    let line = format!(
        "[gd] {{\"location\":\"{}\",\"message\":\"{}\",\"data\":{},\"timestamp\":{}}}\n",
        debug_escape(location),
        debug_escape(message),
        data,
        ts
    );
    let _ = std::io::stderr().write_all(line.as_bytes());
}

// -------- Typed index newtypes (checked constructors, invalid-state prevention) --------

/// Valid file index into `file_starts`. Construct via `FileIx::new(idx, file_count)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct FileIx(usize);

impl FileIx {
    /// Returns `Some(FileIx)` if `idx < file_count`, else `None`.
    pub fn new(idx: usize, file_count: usize) -> Option<Self> {
        if idx < file_count {
            Some(FileIx(idx))
        } else {
            None
        }
    }
    pub fn get(self) -> usize {
        self.0
    }
}

/// Valid line index into `lines` / `line_map`. Construct via `LineIx::new(idx, line_count)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub(crate) struct LineIx(usize);

#[allow(dead_code)]
impl LineIx {
    /// Returns `Some(LineIx)` if `idx < line_count`, else `None`.
    pub fn new(idx: usize, line_count: usize) -> Option<Self> {
        if idx < line_count {
            Some(LineIx(idx))
        } else {
            None
        }
    }
    pub fn get(self) -> usize {
        self.0
    }
}

/// Valid tree entry index into `tree_entries`. Construct via `TreeEntryIx::new(idx, entry_count)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TreeEntryIx(usize);

impl TreeEntryIx {
    /// Returns `Some(TreeEntryIx)` if `idx < entry_count`, else `None`.
    pub fn new(idx: usize, entry_count: usize) -> Option<Self> {
        if idx < entry_count {
            Some(TreeEntryIx(idx))
        } else {
            None
        }
    }
    pub fn get(self) -> usize {
        self.0
    }
}

// -------- Typed state enums (replace boolean meshes) --------

/// Which panel has keyboard focus.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Focus {
    Diff,
    Tree,
}

/// Overlay mode with typed payloads. Used in later chunks.
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub(crate) enum Overlay {
    None,
    Search(SearchState),
    Help,
    Visual(VisualState),
}

/// Search overlay state. Used in later chunks.
#[derive(Debug, Clone, PartialEq, Default)]
#[allow(dead_code)]
pub(crate) struct SearchState {
    pub query: String,
    pub matches: Vec<usize>,
    pub current_match: isize,
    pub input: String,
    pub cursor: usize,
}

/// Visual mode overlay state. Used in later chunks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub(crate) struct VisualState {
    pub anchor: usize,
}

/// View scope: all files or single file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ViewScope {
    AllFiles,
    SingleFile(FileIx),
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Mode {
    Normal,
    Search,
    Help,
    Visual,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum KeyResult {
    Continue,
    ReRender,
    ReGenerate,
    Quit,
    OpenEditor { path: String, lineno: Option<u32> },
}

// -------- Keymap single source (chunk-04) --------

/// Context in which a keybinding applies.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum KeyContext {
    Normal,
    Tree,
    Search,
    Visual,
}

/// Action identifiers. Single source of truth for key→action mapping.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ActionId {
    Quit,
    ScrollDown,
    ScrollUp,
    HalfPageDown,
    HalfPageUp,
    Top,
    Bottom,
    NextHunk,
    PrevHunk,
    NextFile,
    PrevFile,
    ToggleSingleFile,
    ToggleFullContext,
    Search,
    SearchSubmit,
    SearchCancel,
    NextMatch,
    PrevMatch,
    ToggleTree,
    FocusTree,
    FocusTreeOrShow,
    ReturnToDiff,
    TreeClose,
    TreeFirst,
    TreeLast,
    TreeNavDown,
    TreeNavUp,
    TreeSelect,
    EnterVisual,
    VisualExtendDown,
    VisualExtendUp,
    VisualCopy,
    VisualCancel,
    OpenEditor,
    Help,
}

/// Help group for overlay layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum HelpGroup {
    Navigation,
    DiffNav,
    Search,
    FileTree,
    VisualMode,
    Other,
}

/// Keymap entry: keys that trigger an action in a context, plus help display.
#[allow(dead_code)]
struct KeymapEntry {
    action: ActionId,
    context: KeyContext,
    keys: &'static [Key],
    group: HelpGroup,
    key_display: &'static str,
    label: &'static str,
}

fn keymap_entries() -> &'static [KeymapEntry] {
    use ActionId::*;
    use HelpGroup::*;
    use KeyContext::*;
    static ENTRIES: &[KeymapEntry] = &[
        KeymapEntry { action: ScrollDown, context: Normal, keys: &[Key::Char('j'), Key::Down, Key::Enter], group: Navigation, key_display: "j/\u{2193}/Enter", label: "Scroll down" },
        KeymapEntry { action: ScrollUp, context: Normal, keys: &[Key::Char('k'), Key::Up], group: Navigation, key_display: "k/\u{2191}", label: "Scroll up" },
        KeymapEntry { action: HalfPageDown, context: Normal, keys: &[Key::CtrlD, Key::PageDown], group: Navigation, key_display: "Ctrl-D", label: "Half page down" },
        KeymapEntry { action: HalfPageUp, context: Normal, keys: &[Key::CtrlU, Key::PageUp], group: Navigation, key_display: "Ctrl-U", label: "Half page up" },
        KeymapEntry { action: Top, context: Normal, keys: &[Key::Char('g'), Key::Home], group: Navigation, key_display: "g/Home", label: "Top" },
        KeymapEntry { action: Bottom, context: Normal, keys: &[Key::Char('G'), Key::End], group: Navigation, key_display: "G/End", label: "Bottom" },
        KeymapEntry { action: NextHunk, context: Normal, keys: &[Key::Char('d')], group: DiffNav, key_display: "d", label: "Next hunk" },
        KeymapEntry { action: PrevHunk, context: Normal, keys: &[Key::Char('u')], group: DiffNav, key_display: "u", label: "Previous hunk" },
        KeymapEntry { action: NextFile, context: Normal, keys: &[Key::Char('D')], group: DiffNav, key_display: "D", label: "Next file" },
        KeymapEntry { action: PrevFile, context: Normal, keys: &[Key::Char('U')], group: DiffNav, key_display: "U", label: "Previous file" },
        KeymapEntry { action: ToggleSingleFile, context: Normal, keys: &[Key::Char('a')], group: DiffNav, key_display: "a", label: "Toggle single file" },
        KeymapEntry { action: ToggleFullContext, context: Normal, keys: &[Key::Char('z')], group: DiffNav, key_display: "z", label: "Toggle full file context" },
        KeymapEntry { action: ActionId::Search, context: Normal, keys: &[Key::Char('/')], group: HelpGroup::Search, key_display: "/", label: "Search" },
        KeymapEntry { action: SearchSubmit, context: KeyContext::Search, keys: &[Key::Enter], group: HelpGroup::Search, key_display: "Enter", label: "Apply search" },
        KeymapEntry { action: SearchCancel, context: KeyContext::Search, keys: &[Key::Escape, Key::CtrlC], group: HelpGroup::Search, key_display: "Esc", label: "Cancel search" },
        KeymapEntry { action: NextMatch, context: Normal, keys: &[Key::Char('n')], group: HelpGroup::Search, key_display: "n", label: "Next match" },
        KeymapEntry { action: PrevMatch, context: Normal, keys: &[Key::Char('N')], group: HelpGroup::Search, key_display: "N", label: "Previous match" },
        KeymapEntry { action: ToggleTree, context: Normal, keys: &[Key::Char('e')], group: FileTree, key_display: "e", label: "Toggle tree panel" },
        KeymapEntry { action: FocusTree, context: Normal, keys: &[Key::Tab], group: FileTree, key_display: "Tab", label: "Focus panel" },
        KeymapEntry { action: FocusTreeOrShow, context: Normal, keys: &[Key::Char('1')], group: FileTree, key_display: "1", label: "Toggle tree focus" },
        KeymapEntry { action: FocusTreeOrShow, context: Normal, keys: &[Key::CtrlL, Key::Char('l')], group: FileTree, key_display: "l/Ctrl-L", label: "Show + focus tree" },
        KeymapEntry { action: ReturnToDiff, context: Tree, keys: &[Key::CtrlH, Key::Escape, Key::Tab, Key::Char('1'), Key::Char('h')], group: FileTree, key_display: "h/Ctrl-H", label: "Return to diff" },
        KeymapEntry { action: TreeClose, context: Tree, keys: &[Key::Char('e')], group: FileTree, key_display: "e", label: "Close tree" },
        KeymapEntry { action: TreeFirst, context: Tree, keys: &[Key::Char('g'), Key::Home], group: FileTree, key_display: "g/Home", label: "First file" },
        KeymapEntry { action: TreeLast, context: Tree, keys: &[Key::Char('G'), Key::End], group: FileTree, key_display: "G/End", label: "Last file" },
        KeymapEntry { action: TreeNavDown, context: Tree, keys: &[Key::Char('j'), Key::Down], group: FileTree, key_display: "j/k", label: "(tree) Navigate" },
        KeymapEntry { action: TreeNavUp, context: Tree, keys: &[Key::Char('k'), Key::Up], group: FileTree, key_display: "j/k", label: "(tree) Navigate" },
        KeymapEntry { action: TreeSelect, context: Tree, keys: &[Key::Enter], group: FileTree, key_display: "Enter", label: "Select / toggle folder" },
        KeymapEntry { action: ToggleSingleFile, context: Tree, keys: &[Key::Char('a')], group: FileTree, key_display: "a", label: "Toggle single file" },
        KeymapEntry { action: NextHunk, context: Tree, keys: &[Key::Char('d')], group: FileTree, key_display: "d", label: "Next hunk" },
        KeymapEntry { action: PrevHunk, context: Tree, keys: &[Key::Char('u')], group: FileTree, key_display: "u", label: "Previous hunk" },
        KeymapEntry { action: Quit, context: Tree, keys: &[Key::Char('q'), Key::CtrlC], group: FileTree, key_display: "q", label: "Quit" },
        KeymapEntry { action: EnterVisual, context: Normal, keys: &[Key::Char('v')], group: VisualMode, key_display: "v", label: "Enter visual mode" },
        KeymapEntry { action: VisualExtendDown, context: Visual, keys: &[Key::Char('j'), Key::Down], group: VisualMode, key_display: "j/k", label: "Extend selection" },
        KeymapEntry { action: VisualExtendUp, context: Visual, keys: &[Key::Char('k'), Key::Up], group: VisualMode, key_display: "j/k", label: "Extend selection" },
        KeymapEntry { action: VisualCopy, context: Visual, keys: &[Key::Char('y')], group: VisualMode, key_display: "y", label: "Copy path:lines" },
        KeymapEntry { action: VisualCancel, context: Visual, keys: &[Key::Escape, Key::CtrlC], group: VisualMode, key_display: "Esc", label: "Cancel" },
        KeymapEntry { action: Quit, context: Visual, keys: &[Key::Char('q')], group: VisualMode, key_display: "q", label: "Quit" },
        KeymapEntry { action: OpenEditor, context: Normal, keys: &[Key::Char('E')], group: Other, key_display: "E", label: "Open in editor" },
        KeymapEntry { action: Quit, context: Normal, keys: &[Key::Char('q'), Key::CtrlC], group: Other, key_display: "q", label: "Quit" },
        KeymapEntry { action: Help, context: Normal, keys: &[Key::Char('?')], group: Other, key_display: "? / Esc", label: "Close help" },
    ];
    ENTRIES
}

fn keymap_lookup(key: Key, context: KeyContext) -> Option<ActionId> {
    for e in keymap_entries() {
        if e.context == context && e.keys.contains(&key) {
            return Some(e.action);
        }
    }
    None
}

/// Build help lines from keymap. Preserves grouping (Navigation, Diff Nav, Search, File Tree, Visual, Other).
/// Returns raw lines for format_help_lines to pad/center.
fn keymap_help_lines() -> Vec<String> {
    use HelpGroup::*;
    use std::collections::HashSet;
    let order = [Navigation, DiffNav, Search, FileTree, VisualMode, Other];
    let mut lines: Vec<String> = Vec::new();
    for group in order {
        let mut seen: HashSet<(&'static str, &'static str)> = HashSet::new();
        let mut group_lines: Vec<(&'static str, &'static str)> = Vec::new();
        for e in keymap_entries() {
            if e.group == group && !e.keys.is_empty() && !e.key_display.is_empty() {
                let k = (e.key_display, e.label);
                if !seen.contains(&k) {
                    seen.insert(k);
                    group_lines.push((e.key_display, e.label));
                }
            }
        }
        if !group_lines.is_empty() {
            if !lines.is_empty() {
                lines.push(String::new());
            }
            let group_name = match group {
                Navigation => "Navigation",
                DiffNav => "Diff Navigation",
                Search => "Search",
                FileTree => "File Tree",
                VisualMode => "Visual Mode",
                Other => "Other",
            };
            lines.push(group_name.to_string());
            for (k, l) in group_lines {
                let pad = 12usize.saturating_sub(k.chars().count());
                lines.push(format!("{}{}  {}", k, " ".repeat(pad), l));
            }
        }
    }
    lines
}

/// Dispatch by action id. Returns Some(effect) to return immediately, None to fall through to
/// enforce_scrolloff/sync (for scroll-like actions that don't early-return).
fn dispatch_normal_action(
    state: &mut PagerState,
    action: ActionId,
    ctx: &ReducerCtx<'_>,
) -> Option<ReducerEffect> {
    use ActionId::*;
    let ch = ctx.content_height;
    let rows = ctx.rows;
    let files = ctx.files;
    let half_page = ch / 2;
    let (range_start, range_end) = visible_range(state);
    let max_cursor = range_end.saturating_sub(1);

    match action {
        Quit => return Some(ReducerEffect::Quit),
        ScrollDown => {
            let next = (state.cursor_line + 1).min(max_cursor);
            state.cursor_line = next_content_line(&state.doc.line_map, next, max_cursor);
            return None; // fall through
        }
        ScrollUp => {
            let prev = state.cursor_line.saturating_sub(1).max(range_start);
            state.cursor_line = prev_content_line(&state.doc.line_map, prev, range_start);
            return None;
        }
        HalfPageDown => {
            let target = (state.cursor_line + half_page).min(max_cursor);
            state.cursor_line = next_content_line(&state.doc.line_map, target, max_cursor);
            if !is_content_line(&state.doc.line_map, state.cursor_line) {
                state.cursor_line = prev_content_line(&state.doc.line_map, target, range_start);
            }
            return None;
        }
        HalfPageUp => {
            let target = state.cursor_line.saturating_sub(half_page).max(range_start);
            state.cursor_line = prev_content_line(&state.doc.line_map, target, range_start);
            if !is_content_line(&state.doc.line_map, state.cursor_line) {
                state.cursor_line = next_content_line(&state.doc.line_map, target, max_cursor);
            }
            return None;
        }
        Top => {
            state.cursor_line = next_content_line(&state.doc.line_map, range_start, max_cursor);
            return None;
        }
        Bottom => {
            state.cursor_line = prev_content_line(&state.doc.line_map, max_cursor, range_start);
            return None;
        }
        NextHunk => {
            let res = nav_du_down(state, ch);
            state.cursor_line = res.cursor_line;
            state.status_message = res.status_message.clone();
            sync_active_file_to_cursor(state);
            if res.moved {
                let (rs, _, _, max_top) = viewport_bounds(state, ch);
                state.top_line = recenter_top_line(state.cursor_line, ch, rs, max_top);
            }
            sync_tree_cursor(state, ch);
            return Some(ReducerEffect::Continue);
        }
        PrevHunk => {
            let res = nav_du_up(state, ch);
            state.cursor_line = res.cursor_line;
            state.status_message = res.status_message.clone();
            sync_active_file_to_cursor(state);
            if res.moved {
                let (rs, _, _, max_top) = viewport_bounds(state, ch);
                state.top_line = recenter_top_line(state.cursor_line, ch, rs, max_top);
            }
            sync_tree_cursor(state, ch);
            return Some(ReducerEffect::Continue);
        }
        NextFile => {
            let res = nav_D_down(state, ch);
            state.cursor_line = res.cursor_line;
            state.top_line = res.top_line;
            state.status_message = res.status_message;
            sync_tree_cursor(state, ch);
            return Some(ReducerEffect::Continue);
        }
        PrevFile => {
            let res = nav_U_up(state, ch);
            state.cursor_line = res.cursor_line;
            state.top_line = res.top_line;
            state.status_message = res.status_message;
            sync_tree_cursor(state, ch);
            return Some(ReducerEffect::Continue);
        }
        ToggleSingleFile => {
            if state.active_file().is_some() {
                state.set_active_file(None);
                state.status_message = "All files".into();
            } else {
                let anchor = state.cursor_line;
                let file_idx = state.doc.line_map.get(anchor).map_or(0, |li| li.file_idx);
                if state.tree_entries.is_empty() {
                    state.tree_entries = build_tree_entries(files);
                    state.tree_width = compute_tree_width(&state.tree_entries);
                }
                state.set_active_file(Some(file_idx));
                if let Some(start) = state.file_start(file_idx) {
                    let file_end = state.file_end(file_idx).saturating_sub(1);
                    state.top_line = start;
                    state.cursor_line =
                        snap_to_content(&state.doc.line_map, state.top_line, start, file_end);
                }
                state.set_tree_cursor(file_idx_to_entry_idx(&state.tree_entries, file_idx));
                let (tl, tv) =
                    build_tree_lines(&state.tree_entries, state.tree_cursor(), state.tree_width);
                state.tree_lines = tl;
                state.tree_visible_to_entry = tv;
                state.status_message = "Single file".into();
            }
            return Some(ReducerEffect::ReRender);
        }
        ToggleFullContext => {
            state.full_context = !state.full_context;
            state.status_message = if state.full_context {
                "Full file context".into()
            } else {
                "Hunk context".into()
            };
            return Some(ReducerEffect::ReGenerate);
        }
        ActionId::Search => {
            state.mode = Mode::Search;
            return None;
        }
        NextMatch => {
            if !state.search_matches.is_empty() {
                if state.active_file().is_some() {
                    let (rs, re) = visible_range(state);
                    let filtered: Vec<usize> = state
                        .search_matches
                        .iter()
                        .copied()
                        .filter(|&m| m >= rs && m < re)
                        .collect();
                    if !filtered.is_empty() {
                        let cur_line = if state.current_match >= 0 {
                            state.search_matches[state.current_match as usize]
                        } else {
                            0
                        };
                        if let Some(pos) = filtered.iter().position(|&m| m > cur_line) {
                            let global = state
                                .search_matches
                                .iter()
                                .position(|&m| m == filtered[pos])
                                .unwrap();
                            state.current_match = global as isize;
                        } else {
                            let global = state
                                .search_matches
                                .iter()
                                .position(|&m| m == filtered[0])
                                .unwrap();
                            state.current_match = global as isize;
                        }
                        scroll_to_match(state, rows);
                    }
                } else {
                    state.current_match =
                        (state.current_match + 1) % state.search_matches.len() as isize;
                    scroll_to_match(state, rows);
                }
            }
            return None;
        }
        PrevMatch => {
            if !state.search_matches.is_empty() {
                if state.active_file().is_some() {
                    let (rs, re) = visible_range(state);
                    let filtered: Vec<usize> = state
                        .search_matches
                        .iter()
                        .copied()
                        .filter(|&m| m >= rs && m < re)
                        .collect();
                    if !filtered.is_empty() {
                        let cur_line = if state.current_match >= 0 {
                            state.search_matches[state.current_match as usize]
                        } else {
                            usize::MAX
                        };
                        if let Some(pos) = filtered.iter().rposition(|&m| m < cur_line) {
                            let global = state
                                .search_matches
                                .iter()
                                .position(|&m| m == filtered[pos])
                                .unwrap();
                            state.current_match = global as isize;
                        } else {
                            let last = *filtered.last().unwrap();
                            let global = state
                                .search_matches
                                .iter()
                                .position(|&m| m == last)
                                .unwrap();
                            state.current_match = global as isize;
                        }
                        scroll_to_match(state, rows);
                    }
                } else {
                    state.current_match = (state.current_match - 1
                        + state.search_matches.len() as isize)
                        % state.search_matches.len() as isize;
                    scroll_to_match(state, rows);
                }
            }
            return None;
        }
        ToggleTree => {
            if state.tree_visible && state.tree_focused() {
                state.tree_visible = false;
                state.set_tree_focused(false);
            } else if state.tree_visible && !state.tree_focused() {
                state.set_tree_focused(true);
                let anchor = state.cursor_line;
                let file_idx = state.doc.line_map.get(anchor).map_or(0, |li| li.file_idx);
                state.set_tree_cursor(file_idx_to_entry_idx(&state.tree_entries, file_idx));
                let (tl, tv) =
                    build_tree_lines(&state.tree_entries, state.tree_cursor(), state.tree_width);
                state.tree_lines = tl;
                state.tree_visible_to_entry = tv;
                ensure_tree_cursor_visible(state, ch);
            } else if !state.tree_visible {
                state.tree_visible = true;
                state.set_tree_focused(true);
                let anchor = state.cursor_line;
                let file_idx = state.doc.line_map.get(anchor).map_or(0, |li| li.file_idx);
                state.tree_entries = build_tree_entries(files);
                state.tree_width = compute_tree_width(&state.tree_entries);
                state.set_tree_cursor(file_idx_to_entry_idx(&state.tree_entries, file_idx));
                ensure_tree_cursor_visible(state, ch);
            }
            return Some(ReducerEffect::ReRender);
        }
        FocusTree => {
            if state.tree_visible {
                let anchor = state.cursor_line;
                let file_idx = state.doc.line_map.get(anchor).map_or(0, |li| li.file_idx);
                state.set_tree_cursor(file_idx_to_entry_idx(&state.tree_entries, file_idx));
                let (tl, tv) =
                    build_tree_lines(&state.tree_entries, state.tree_cursor(), state.tree_width);
                state.tree_lines = tl;
                state.tree_visible_to_entry = tv;
                state.set_tree_focused(true);
                ensure_tree_cursor_visible(state, ch);
            }
            return None;
        }
        FocusTreeOrShow => {
            if !state.tree_visible {
                state.tree_visible = true;
                let anchor = state.cursor_line;
                let file_idx = state.doc.line_map.get(anchor).map_or(0, |li| li.file_idx);
                state.tree_entries = build_tree_entries(files);
                state.tree_width = compute_tree_width(&state.tree_entries);
                state.set_tree_cursor(file_idx_to_entry_idx(&state.tree_entries, file_idx));
            }
            state.set_tree_focused(true);
            ensure_tree_cursor_visible(state, ch);
            return Some(ReducerEffect::ReRender);
        }
        EnterVisual => {
            state.mode = Mode::Visual;
            state.visual_anchor = state.cursor_line;
            return None;
        }
        OpenEditor => {
            let pos = state.cursor_line.min(state.doc.line_map.len().saturating_sub(1));
            if !state.doc.line_map.is_empty() {
                let info = &state.doc.line_map[pos];
                let path = info.path.clone();
                let lineno = info.new_lineno;
                return Some(ReducerEffect::OpenEditor { path, lineno });
            }
            return None;
        }
        Help => {
            state.mode = Mode::Help;
            return None;
        }
        ReturnToDiff | TreeClose | TreeFirst | TreeLast | TreeNavDown | TreeNavUp
        | TreeSelect | VisualExtendDown | VisualExtendUp | VisualCopy
        | VisualCancel | SearchSubmit | SearchCancel => {
            // Not valid in Normal context; keymap lookup should not return these for Normal
            return None;
        }
    }
}

fn dispatch_tree_action(
    state: &mut PagerState,
    action: ActionId,
    ctx: &ReducerCtx<'_>,
) -> ReducerEffect {
    use ActionId::*;
    let ch = ctx.content_height;

    match action {
        TreeNavDown => {
            let _ = move_tree_selection(state, 1, ch);
        }
        TreeNavUp => {
            let _ = move_tree_selection(state, -1, ch);
        }
        TreeSelect => {
            let cursor = state.tree_cursor();
            if let Some(entry) = state.tree_entry(cursor) {
                if entry.file_idx.is_none() {
                    if let Some(e) = state.tree_entry_mut(cursor) {
                        e.collapsed = !e.collapsed;
                    }
                    let (tl, tv) =
                        build_tree_lines(&state.tree_entries, state.tree_cursor(), state.tree_width);
                    state.tree_lines = tl;
                    state.tree_visible_to_entry = tv;
                    ensure_tree_cursor_visible(state, ch);
                } else if let Some(fi) = entry.file_idx {
                    if let Some(target) = state.file_start(fi) {
                        if state.active_file().is_some() {
                            state.set_active_file(Some(fi));
                        }
                        state.top_line = target;
                        let file_end = state.file_end(fi).saturating_sub(1);
                        state.cursor_line =
                            snap_to_content(&state.doc.line_map, state.top_line, target, file_end);
                    }
                }
            }
        }
        ReturnToDiff => {
            state.set_tree_focused(false);
        }
        TreeClose => {
            state.tree_visible = false;
            state.set_tree_focused(false);
            return ReducerEffect::ReRender;
        }
        TreeFirst => {
            if let Some(&first) = state.tree_visible_to_entry.first() {
                state.set_tree_cursor(first);
                if let Some(fi) = state.tree_entry(first).and_then(|e| e.file_idx) {
                    if let Some(start) = state.file_start(fi) {
                        let file_end = state.file_end(fi).saturating_sub(1);
                        state.top_line = start;
                        state.cursor_line =
                            snap_to_content(&state.doc.line_map, state.top_line, start, file_end);
                    }
                }
                let (tl, tv) =
                    build_tree_lines(&state.tree_entries, state.tree_cursor(), state.tree_width);
                state.tree_lines = tl;
                state.tree_visible_to_entry = tv;
                ensure_tree_cursor_visible(state, ch);
            }
        }
        TreeLast => {
            if let Some(&last) = state.tree_visible_to_entry.last() {
                state.set_tree_cursor(last);
                if let Some(fi) = state.tree_entry(last).and_then(|e| e.file_idx) {
                    if let Some(start) = state.file_start(fi) {
                        let file_end = state.file_end(fi).saturating_sub(1);
                        state.top_line = start;
                        state.cursor_line =
                            snap_to_content(&state.doc.line_map, state.top_line, start, file_end);
                    }
                }
                let (tl, tv) =
                    build_tree_lines(&state.tree_entries, state.tree_cursor(), state.tree_width);
                state.tree_lines = tl;
                state.tree_visible_to_entry = tv;
                ensure_tree_cursor_visible(state, ch);
            }
        }
        ToggleSingleFile => {
            if state.active_file().is_some() {
                state.set_active_file(None);
                state.status_message = "All files".into();
            } else {
                let file_idx = state
                    .visible_tree_entry()
                    .and_then(|e| e.file_idx)
                    .unwrap_or(0);
                state.set_active_file(Some(file_idx));
                if let Some(start) = state.file_start(file_idx) {
                    let file_end = state.file_end(file_idx).saturating_sub(1);
                    state.top_line = start;
                    state.cursor_line =
                        snap_to_content(&state.doc.line_map, state.top_line, start, file_end);
                }
                state.set_tree_cursor(file_idx_to_entry_idx(&state.tree_entries, file_idx));
                let (tl, tv) = build_tree_lines(&state.tree_entries, state.tree_cursor(), state.tree_width);
                state.tree_lines = tl;
                state.tree_visible_to_entry = tv;
                state.status_message = "Single file".into();
            }
            return ReducerEffect::ReRender;
        }
        NextHunk => {
            let res = nav_du_down(state, ch);
            state.cursor_line = res.cursor_line;
            state.status_message = res.status_message.clone();
            sync_active_file_to_cursor(state);
            if res.moved {
                let (range_start, _, _, max_top) = viewport_bounds(state, ch);
                state.top_line = recenter_top_line(state.cursor_line, ch, range_start, max_top);
            }
            sync_tree_cursor_force(state, ch);
        }
        PrevHunk => {
            let res = nav_du_up(state, ch);
            state.cursor_line = res.cursor_line;
            state.status_message = res.status_message.clone();
            sync_active_file_to_cursor(state);
            if res.moved {
                let (range_start, _, _, max_top) = viewport_bounds(state, ch);
                state.top_line = recenter_top_line(state.cursor_line, ch, range_start, max_top);
            }
            sync_tree_cursor_force(state, ch);
        }
        Quit => return ReducerEffect::Quit,
        _ => {}
    }
    ReducerEffect::Continue
}

fn dispatch_visual_action(
    state: &mut PagerState,
    action: ActionId,
    ctx: &ReducerCtx<'_>,
) -> ReducerEffect {
    use ActionId::*;
    let ch = ctx.content_height;

    match action {
        VisualExtendDown => {
            let next = state.cursor_line + 1;
            let anchor_file = state
                .doc.line_map
                .get(state.visual_anchor)
                .map_or(0, |l| l.file_idx);
            let next_file = state
                .doc.line_map
                .get(next)
                .map_or(usize::MAX, |l| l.file_idx);
            if next < state.doc.lines.len() && next_file == anchor_file {
                state.cursor_line = next;
                if state.cursor_line >= state.top_line + ch {
                    state.top_line = state.cursor_line - ch + 1;
                }
            }
        }
        VisualExtendUp => {
            if state.cursor_line > 0 {
                let prev = state.cursor_line - 1;
                let anchor_file = state
                    .doc.line_map
                    .get(state.visual_anchor)
                    .map_or(0, |l| l.file_idx);
                let prev_file = state
                    .doc.line_map
                    .get(prev)
                    .map_or(usize::MAX, |l| l.file_idx);
                if prev_file == anchor_file {
                    state.cursor_line = prev;
                    if state.cursor_line < state.top_line {
                        state.top_line = state.cursor_line;
                    }
                }
            }
        }
        VisualCopy => {
            let lo = state.visual_anchor.min(state.cursor_line);
            let hi = state.visual_anchor.max(state.cursor_line);
            let path = state
                .doc.line_map
                .get(lo)
                .map(|l| l.path.clone())
                .unwrap_or_default();
            let (start, end) = resolve_lineno(&state.doc.line_map, lo, hi);
            let text = format_copy_ref(&path, start, end);
            let ok = copy_to_clipboard(&text);
            state.status_message = if ok {
                format!("Copied: {text}")
            } else {
                "Copy failed (pbcopy not available)".to_string()
            };
            state.mode = Mode::Normal;
            state.cursor_line = state.top_line;
            let (rs, re) = visible_range(state);
            state.cursor_line = snap_to_content(
                &state.doc.line_map,
                state.cursor_line,
                rs,
                re.saturating_sub(1),
            );
            enforce_scrolloff(state, ch);
            sync_tree_cursor(state, ch);
        }
        VisualCancel => {
            state.mode = Mode::Normal;
            state.cursor_line = state.top_line;
            let (rs, re) = visible_range(state);
            state.cursor_line = snap_to_content(
                &state.doc.line_map,
                state.cursor_line,
                rs,
                re.saturating_sub(1),
            );
        }
        Quit => return ReducerEffect::Quit,
        _ => {}
    }
    ReducerEffect::Continue
}

// -------- Reducer boundary (chunk-02) --------

/// Input event for the reducer. Keys map to events; mode-specific handlers process them.
#[derive(Debug, Clone)]
pub(crate) enum ReducerEvent {
    Key(Key),
}

/// Effect emitted by the reducer. Mirrors KeyResult.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum ReducerEffect {
    Continue,
    ReRender,
    ReGenerate,
    Quit,
    OpenEditor { path: String, lineno: Option<u32> },
}

impl From<ReducerEffect> for KeyResult {
    fn from(e: ReducerEffect) -> Self {
        match e {
            ReducerEffect::Continue => KeyResult::Continue,
            ReducerEffect::ReRender => KeyResult::ReRender,
            ReducerEffect::ReGenerate => KeyResult::ReGenerate,
            ReducerEffect::Quit => KeyResult::Quit,
            ReducerEffect::OpenEditor { path, lineno } => KeyResult::OpenEditor { path, lineno },
        }
    }
}

/// Context passed into the reducer (content height, total rows, files for tree/editor).
#[derive(Debug)]
pub(crate) struct ReducerCtx<'a> {
    pub content_height: usize,
    pub rows: u16,
    pub files: &'a [DiffFile],
}

/// Invariant guard: cursor/top in range, tree focus valid, single-file scope valid.
#[cfg(debug_assertions)]
fn debug_assert_valid_state(state: &PagerState) {
    let (rs, re) = visible_range(state);
    let max_cursor = re.saturating_sub(1);
    assert!(
        state.cursor_line >= rs && state.cursor_line <= max_cursor,
        "cursor_line {} out of visible range [{}, {}]",
        state.cursor_line,
        rs,
        max_cursor
    );
    assert!(
        state.top_line >= rs && state.top_line <= re.saturating_sub(1).min(state.doc.line_count().saturating_sub(1)),
        "top_line {} out of range",
        state.top_line
    );
    if state.tree_focused() {
        assert!(!state.tree_entries.is_empty(), "tree focus invalid when tree empty");
        assert!(
            state.tree_selection.is_some() && state.tree_selection.unwrap().get() < state.tree_entry_count(),
            "tree_selection invalid"
        );
    }
    if let ViewScope::SingleFile(ix) = state.view_scope {
        assert!(ix.get() < state.file_count(), "SingleFile index {} out of bounds", ix.get());
    }
}

#[cfg(not(debug_assertions))]
fn debug_assert_valid_state(_state: &PagerState) {}

/// Clamp cursor and top_line to visible range. Call before debug_assert_valid_state.
fn clamp_cursor_and_top(state: &mut PagerState) {
    let (rs, re) = visible_range(state);
    let range_max = re.saturating_sub(1);
    let max_top = range_max.saturating_sub(1).min(state.doc.line_count().saturating_sub(1));
    state.cursor_line = state.cursor_line.clamp(rs, range_max);
    state.top_line = state.top_line.clamp(rs, max_top.min(range_max));
}

pub struct DiffContext {
    pub repo: std::path::PathBuf,
    pub source: crate::git::DiffSource,
    pub no_untracked: bool,
}

// -------- Immutable document model (chunk-03) --------

/// Immutable document encapsulating all rendered diff content. Swapped atomically
/// on regenerate/rerender/resize; view state (cursor, top, scope) is remapped via remap_after_document_swap.
#[derive(Debug, Clone)]
pub(crate) struct Document {
    pub(crate) lines: Vec<String>,
    pub(crate) line_map: Vec<LineInfo>,
    pub(crate) file_starts: Vec<usize>,
    pub(crate) hunk_starts: Vec<usize>,
}

impl Document {
    pub(crate) fn from_render_output(output: RenderOutput) -> Self {
        Document {
            lines: output.lines,
            line_map: output.line_map,
            file_starts: output.file_starts,
            hunk_starts: output.hunk_starts,
        }
    }

    pub(crate) fn line_count(&self) -> usize {
        self.lines.len()
    }

    pub(crate) fn file_count(&self) -> usize {
        self.file_starts.len()
    }

    pub(crate) fn file_start(&self, idx: usize) -> Option<usize> {
        self.file_starts.get(idx).copied()
    }

    pub(crate) fn file_end(&self, idx: usize) -> usize {
        self.file_starts
            .get(idx + 1)
            .copied()
            .unwrap_or(self.lines.len())
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.line_map.is_empty()
    }
}

/// Anchor for preserving view position across document swaps. Captured before swap,
/// used to remap cursor/top to nearest valid content in the new document.
#[derive(Debug, Clone)]
struct ViewAnchor {
    file_idx: usize,
    new_lineno: Option<u32>,
    offset_in_file: usize,
}

#[derive(Debug)]
pub(crate) struct PagerState {
    pub(crate) doc: Document,
    pub(crate) top_line: usize,
    pub(crate) cursor_line: usize,
    pub(crate) visual_anchor: usize,
    pub(crate) search_query: String,
    pub(crate) search_matches: Vec<usize>,
    pub(crate) current_match: isize,
    pub(crate) mode: Mode,
    pub(crate) search_input: String,
    pub(crate) search_cursor: usize,
    pub(crate) status_message: String,
    pub(crate) tree_visible: bool,
    /// Typed focus: Diff = diff panel focused, Tree = tree panel focused.
    /// Invariant: when tree has no entries, focus must be Diff.
    focus: Focus,
    /// Valid tree selection when tree has entries. None when tree empty.
    tree_selection: Option<TreeEntryIx>,
    pub(crate) tree_width: usize,
    pub(crate) tree_scroll: usize,
    pub(crate) tree_lines: Vec<String>,
    pub(crate) tree_entries: Vec<TreeEntry>,
    /// Maps visible tree line index to original `tree_entries` index
    pub(crate) tree_visible_to_entry: Vec<usize>,
    /// Typed view scope. Invariant: SingleFile(ix) implies ix is valid for file_starts.
    view_scope: ViewScope,
    pub(crate) full_context: bool,
}

impl PagerState {
    pub(crate) fn file_count(&self) -> usize {
        self.doc.file_count()
    }

    #[allow(dead_code)]
    pub(crate) fn line_count(&self) -> usize {
        self.doc.line_count()
    }

    pub(crate) fn tree_entry_count(&self) -> usize {
        self.tree_entries.len()
    }

    /// Checked accessor for file start line. Returns None if idx is out of bounds.
    pub(crate) fn file_start(&self, idx: usize) -> Option<usize> {
        self.doc.file_start(idx)
    }

    /// Checked accessor for file end line (exclusive). Returns line_count for last file.
    pub(crate) fn file_end(&self, idx: usize) -> usize {
        self.doc.file_end(idx)
    }

    /// Checked accessor for tree entry. Returns None if idx is out of bounds.
    pub(crate) fn tree_entry(&self, idx: usize) -> Option<&TreeEntry> {
        self.tree_entries.get(idx)
    }

    /// Checked mutable accessor for tree entry.
    pub(crate) fn tree_entry_mut(&mut self, idx: usize) -> Option<&mut TreeEntry> {
        self.tree_entries.get_mut(idx)
    }

    /// Start line of the active file, or 0 if all-files view.
    #[allow(dead_code)]
    pub(crate) fn active_file_start(&self) -> usize {
        match self.view_scope {
            ViewScope::AllFiles => 0,
            ViewScope::SingleFile(ix) => self.file_start(ix.get()).unwrap_or(0),
        }
    }

    /// End line (exclusive) of the active file, or line_count if all-files view.
    #[allow(dead_code)]
    pub(crate) fn active_file_end(&self) -> usize {
        match self.view_scope {
            ViewScope::AllFiles => self.doc.line_count(),
            ViewScope::SingleFile(ix) => self.file_end(ix.get()),
        }
    }

    /// Visible tree entry at cursor, if tree has entries and cursor is valid.
    pub(crate) fn visible_tree_entry(&self) -> Option<&TreeEntry> {
        self.tree_selection
            .and_then(|ix| self.tree_entry(ix.get()))
    }

    /// LineInfo at cursor, if in range.
    #[allow(dead_code)]
    pub(crate) fn cursor_line_info(&self) -> Option<&LineInfo> {
        self.doc.line_map.get(self.cursor_line)
    }

    // Compatibility adapters (for migration in later chunks)
    pub(crate) fn active_file(&self) -> Option<usize> {
        match self.view_scope {
            ViewScope::AllFiles => None,
            ViewScope::SingleFile(ix) => Some(ix.get()),
        }
    }

    pub(crate) fn set_active_file(&mut self, v: Option<usize>) {
        self.view_scope = match v {
            None => ViewScope::AllFiles,
            Some(idx) => FileIx::new(idx, self.doc.file_count())
                .map(ViewScope::SingleFile)
                .unwrap_or(ViewScope::AllFiles),
        };
    }

    pub(crate) fn tree_focused(&self) -> bool {
        matches!(self.focus, Focus::Tree)
    }

    pub(crate) fn set_tree_focused(&mut self, focused: bool) {
        self.focus = if focused { Focus::Tree } else { Focus::Diff };
        // Invariant: tree focus invalid when tree empty
        if focused && self.tree_entries.is_empty() {
            self.focus = Focus::Diff;
        }
    }

    pub(crate) fn tree_cursor(&self) -> usize {
        self.tree_selection.map(|s| s.get()).unwrap_or(0)
    }

    pub(crate) fn set_tree_cursor(&mut self, idx: usize) {
        self.tree_selection = TreeEntryIx::new(idx, self.tree_entry_count());
        if self.tree_selection.is_none() && self.focus == Focus::Tree {
            self.focus = Focus::Diff;
        }
    }

    /// Build a valid initial state. Tree starts visible, focused; scope all-files.
    /// Tree panel falls back to hidden when no entries.
    pub(crate) fn new(
        lines: Vec<String>,
        line_map: Vec<LineInfo>,
        file_starts: Vec<usize>,
        hunk_starts: Vec<usize>,
        tree_entries: Vec<TreeEntry>,
    ) -> Self {
        let doc = Document {
            lines,
            line_map,
            file_starts,
            hunk_starts,
        };
        Self::from_doc(doc, tree_entries)
    }

    /// Build state from a Document and tree entries. Used by new() and after document swaps.
    pub(crate) fn from_doc(doc: Document, tree_entries: Vec<TreeEntry>) -> Self {
        let entry_count = tree_entries.len();
        let tree_width = compute_tree_width(&tree_entries);

        let (tree_selection, focus, tree_visible) = if entry_count > 0 {
            let sel = TreeEntryIx::new(0, entry_count).unwrap();
            let (_tl, _tv) = build_tree_lines(&tree_entries, 0, tree_width);
            (Some(sel), Focus::Tree, true)
        } else {
            (None, Focus::Diff, false)
        };

        let (tree_lines, tree_visible_to_entry) = if tree_visible {
            let (tl, tv) = build_tree_lines(&tree_entries, 0, tree_width);
            (tl, tv)
        } else {
            (Vec::new(), Vec::new())
        };

        PagerState {
            doc,
            top_line: 0,
            cursor_line: 0,
            visual_anchor: 0,
            search_query: String::new(),
            search_matches: Vec::new(),
            current_match: -1,
            mode: Mode::Normal,
            search_input: String::new(),
            search_cursor: 0,
            status_message: String::new(),
            tree_visible,
            focus,
            tree_selection,
            tree_width: if tree_visible { tree_width } else { 0 },
            tree_scroll: 0,
            tree_lines,
            tree_entries,
            tree_visible_to_entry,
            view_scope: ViewScope::AllFiles,
            full_context: false,
        }
    }

}

use tui::pager::crossterm_to_key;

fn diff_area_width(cols: u16, tree_width: usize, tree_visible: bool, scrollbar: bool) -> usize {
    let w = if tree_visible {
        (cols as usize).saturating_sub(tree_width + 1)
    } else {
        cols as usize
    };
    if scrollbar { w.saturating_sub(1) } else { w }
}

pub(crate) fn render_scrollbar_cell(
    row: usize,
    content_height: usize,
    vis_start: usize,
    vis_end: usize,
    top: usize,
    line_map: &[LineInfo],
) -> String {
    let range = vis_end - vis_start;
    if range == 0 {
        return format!("{} {}", style::BG_SCROLLBAR_TRACK, style::RESET);
    }

    // Map this screen row to the proportional range of content lines
    let line_start = (vis_start + (row * range) / content_height).min(line_map.len());
    let line_end = (vis_start + ((row + 1) * range) / content_height).min(line_map.len());

    // Scan for change markers in the mapped range
    let mut change: Option<LineKind> = None;
    for li in &line_map[line_start..line_end] {
        if let Some(LineKind::Added | LineKind::Deleted) = li.line_kind {
            change = li.line_kind;
            break;
        }
    }

    // Determine if this row falls within the viewport thumb
    let thumb_start = (top.saturating_sub(vis_start)) * content_height / range;
    let thumb_end = (thumb_start + content_height * content_height / range).max(thumb_start + 1);
    let in_thumb = row >= thumb_start && row < thumb_end;

    let bg = if in_thumb {
        style::BG_SCROLLBAR_THUMB
    } else {
        style::BG_SCROLLBAR_TRACK
    };

    match change {
        Some(LineKind::Added) => {
            format!("{bg}{}\u{2590}{}", style::FG_ADDED_MARKER, style::RESET)
        }
        Some(LineKind::Deleted) => {
            format!("{bg}{}\u{2590}{}", style::FG_DELETED_MARKER, style::RESET)
        }
        _ => {
            format!("{bg} {}", style::RESET)
        }
    }
}

const SCROLLOFF: usize = 8;

fn enforce_scrolloff(state: &mut PagerState, content_height: usize) {
    let (range_start, range_end) = visible_range(state);
    let range_lines = range_end - range_start;
    let max_top = range_start + range_lines.saturating_sub(content_height);
    let max_cursor = range_end.saturating_sub(1);
    state.cursor_line = state.cursor_line.clamp(range_start, max_cursor);
    if state.cursor_line < state.top_line + SCROLLOFF {
        state.top_line = state.cursor_line.saturating_sub(SCROLLOFF).max(range_start);
    }
    if state.cursor_line + SCROLLOFF >= state.top_line + content_height {
        state.top_line = (state.cursor_line + SCROLLOFF + 1).saturating_sub(content_height);
    }
    state.top_line = state.top_line.clamp(range_start, max_top);
}

fn viewport_bounds(state: &PagerState, content_height: usize) -> (usize, usize, usize, usize) {
    let (range_start, range_end) = visible_range(state);
    let max_line = range_end.saturating_sub(1);
    let max_top = range_end.saturating_sub(content_height).max(range_start);
    (range_start, range_end, max_line, max_top)
}

fn recenter_top_line(cursor_line: usize, content_height: usize, range_start: usize, max_top: usize) -> usize {
    cursor_line
        .saturating_sub(content_height / 2)
        .max(range_start)
        .min(max_top)
}

/// Returns true if the line at `idx` is a content line (Added, Deleted, or Context).
pub(crate) fn is_content_line(line_map: &[LineInfo], idx: usize) -> bool {
    line_map.get(idx).is_some_and(|li| li.line_kind.is_some())
}

/// Scan forward from `from` to find the next content line, clamped to `max`.
pub(crate) fn next_content_line(line_map: &[LineInfo], from: usize, max: usize) -> usize {
    let mut i = from;
    while i <= max {
        if is_content_line(line_map, i) {
            return i;
        }
        i += 1;
    }
    from
}

/// Scan backward from `from` to find the previous content line, clamped to `min`.
pub(crate) fn prev_content_line(line_map: &[LineInfo], from: usize, min: usize) -> usize {
    let mut i = from;
    loop {
        if is_content_line(line_map, i) {
            return i;
        }
        if i <= min {
            break;
        }
        i -= 1;
    }
    from
}

/// Snap `pos` to the nearest content line within `[range_start, range_end]`.
/// Tries forward first, then falls back to backward.
pub(crate) fn snap_to_content(
    line_map: &[LineInfo],
    pos: usize,
    range_start: usize,
    range_end: usize,
) -> usize {
    let fwd = next_content_line(line_map, pos, range_end);
    if is_content_line(line_map, fwd) {
        return fwd;
    }
    prev_content_line(line_map, pos, range_start)
}

/// Capture view anchor from current state for remap after document swap.
/// Returns None when document is empty.
fn capture_view_anchor(state: &PagerState) -> Option<ViewAnchor> {
    if state.doc.is_empty() {
        return None;
    }
    let top = state.top_line.min(state.doc.line_map.len().saturating_sub(1));
    let info = &state.doc.line_map[top];
    let file_start = state.file_start(info.file_idx).unwrap_or(0);
    let offset_in_file = top - file_start;
    Some(ViewAnchor {
        file_idx: info.file_idx,
        new_lineno: info.new_lineno,
        offset_in_file,
    })
}

/// Remap cursor/top/scope/tree/overlay after swapping in a new document.
/// Handles file removal and collapsed ranges by falling back to nearest valid content.
fn remap_after_document_swap(
    state: &mut PagerState,
    anchor: Option<ViewAnchor>,
    new_doc: Document,
    files: &[DiffFile],
) {
    state.doc = new_doc;

    // 1. Normalize scope: downgrade single-file to all-files if target file gone
    let file_count = state.doc.file_count();
    state.view_scope = match state.view_scope {
        ViewScope::AllFiles => ViewScope::AllFiles,
        ViewScope::SingleFile(ix) => {
            if ix.get() < file_count {
                ViewScope::SingleFile(ix)
            } else {
                ViewScope::AllFiles
            }
        }
    };

    // 2. Normalize tree selection when tree has entries
    let entry_count = state.tree_entry_count();
    if entry_count > 0 {
        let current = state.tree_cursor();
        let clamped = current.min(entry_count.saturating_sub(1));
        state.set_tree_cursor(clamped);
    } else {
        state.tree_selection = None;
        state.focus = Focus::Diff;
        state.tree_visible_to_entry.clear();
        state.tree_lines.clear();
    }

    // 3. Remap cursor and top from anchor
    let line_count = state.doc.line_count();
    let (rs, re) = visible_range(state);
    let range_max = re.saturating_sub(1);

    if state.doc.is_empty() {
        state.top_line = 0;
        state.cursor_line = 0;
        state.visual_anchor = 0;
    } else if let Some(a) = anchor {
        // Resolve anchor to line index; fall back to nearest valid on file removal/collapsed
        let target_line = if a.file_idx >= file_count {
            // Target file no longer exists: use first content line of doc
            next_content_line(&state.doc.line_map, 0, line_count.saturating_sub(1))
        } else if let Some(lineno) = a.new_lineno {
            state.doc.line_map
                .iter()
                .position(|li| li.file_idx == a.file_idx && li.new_lineno == Some(lineno))
                .unwrap_or_else(|| {
                    let file_start = state.doc.file_start(a.file_idx).unwrap_or(0);
                    let file_end = state.doc.file_end(a.file_idx).saturating_sub(1);
                    (file_start + a.offset_in_file).min(file_end).min(line_count.saturating_sub(1))
                })
        } else {
            let file_start = state.doc.file_start(a.file_idx).unwrap_or(0);
            let file_end = state.doc.file_end(a.file_idx).saturating_sub(1);
            (file_start + a.offset_in_file).min(file_end).min(line_count.saturating_sub(1))
        };
        state.top_line = target_line.min(range_max);
        state.cursor_line = state.top_line.min(range_max);
        state.top_line = state.top_line.clamp(rs, range_max);
        state.cursor_line = state.cursor_line.clamp(rs, range_max);
        state.cursor_line = snap_to_content(&state.doc.line_map, state.cursor_line, rs, range_max);
        state.visual_anchor = state.cursor_line;
    } else {
        state.top_line = 0;
        state.cursor_line = 0;
        state.visual_anchor = 0;
    }

    // 4. Rebuild tree entries and sync tree selection from cursor
    if state.tree_visible && !files.is_empty() {
        state.tree_entries = build_tree_entries(files);
        state.tree_width = compute_tree_width(&state.tree_entries);
        let cursor_file_idx = state
            .doc
            .line_map
            .get(state.cursor_line)
            .map(|li| li.file_idx)
            .unwrap_or(0);
        let cursor_entry_idx = file_idx_to_entry_idx(&state.tree_entries, cursor_file_idx);
        state.set_tree_cursor(cursor_entry_idx);
        let (tl, tv) = build_tree_lines(
            &state.tree_entries,
            state.tree_cursor(),
            state.tree_width,
        );
        state.tree_lines = tl;
        state.tree_visible_to_entry = tv;
    }

    // 5. Re-run search against new lines
    if !state.search_query.is_empty() {
        state.search_matches = find_matches(&state.doc.lines, &state.search_query);
        state.current_match = find_nearest_match(&state.search_matches, state.top_line);
    }

    // 6. Remap/reset overlay payloads: visual anchor already set; search matches updated above.
    // Invalid visual ranges (anchor/cursor out of doc) handled by clamp above.
}

/// Return the `(start, end)` line range for the active file, or the full
/// document range when no file is selected.
pub(crate) fn visible_range(state: &PagerState) -> (usize, usize) {
    match state.active_file() {
        Some(idx) => {
            let start = state.file_start(idx).unwrap_or(0);
            let end = state.file_end(idx);
            (start, end)
        }
        None => (0, state.doc.line_count()),
    }
}

/// Apply visual selection background tint to a rendered line.
/// Prepends BG_VISUAL so the tint shows on gutter/context areas;
/// diff bg codes (BG_ADDED/BG_DELETED) naturally override in content.
fn highlight_visual_line(line: &str, width: usize) -> String {
    let vis_w = crate::ansi::visible_width(line);
    let target = width.saturating_sub(1);
    let pad = target.saturating_sub(vis_w);
    format!(
        "{}{line}{}{}",
        style::BG_VISUAL,
        " ".repeat(pad),
        style::RESET,
    )
}

/// Resolve start/end line numbers from a visual selection range.
/// Uses new_lineno only; falls back to old_lineno only when the entire
/// selection lacks new_lineno (pure deletions).
fn resolve_lineno(line_map: &[LineInfo], lo: usize, hi: usize) -> (Option<u32>, Option<u32>) {
    let new_start = (lo..=hi).find_map(|i| line_map.get(i).and_then(|li| li.new_lineno));
    let new_end = (lo..=hi).rev().find_map(|i| line_map.get(i).and_then(|li| li.new_lineno));
    if new_start.is_some() && new_end.is_some() {
        return (new_start, new_end);
    }
    // Pure deletion selection: fall back to old_lineno
    let old_start = (lo..=hi).find_map(|i| line_map.get(i).and_then(|li| li.old_lineno));
    let old_end = (lo..=hi).rev().find_map(|i| line_map.get(i).and_then(|li| li.old_lineno));
    (old_start, old_end)
}

fn format_copy_ref(path: &str, start: Option<u32>, end: Option<u32>) -> String {
    match (start, end) {
        (Some(s), Some(e)) if s == e => format!("{path}:{s}"),
        (Some(s), Some(e)) => format!("{path}:{s}-{e}"),
        (Some(s), None) => format!("{path}:{s}"),
        _ => path.to_string(),
    }
}

fn prev_char_boundary(s: &str, pos: usize) -> usize {
    let mut pos = pos.min(s.len());
    if pos == 0 {
        return 0;
    }
    pos -= 1;
    while pos > 0 && !s.is_char_boundary(pos) {
        pos -= 1;
    }
    pos
}

fn next_char_boundary(s: &str, pos: usize) -> usize {
    let mut pos = pos.min(s.len());
    if pos == s.len() {
        return pos;
    }
    pos += 1;
    while pos < s.len() && !s.is_char_boundary(pos) {
        pos += 1;
    }
    pos
}

fn clamp_cursor_to_boundary(s: &str, cursor: usize) -> usize {
    let mut cursor = cursor.min(s.len());
    while cursor > 0 && !s.is_char_boundary(cursor) {
        cursor -= 1;
    }
    cursor
}

fn submit_search(state: &mut PagerState) {
    let query = std::mem::take(&mut state.search_input);
    state.search_cursor = 0;
    state.mode = Mode::Normal;

    let matches = find_matches(&state.doc.lines, &query);
    if matches.is_empty() {
        state.status_message = format!("Pattern not found: {query}");
        state.search_query = query;
        state.search_matches = Vec::new();
        state.current_match = -1;
    } else {
        let nearest = find_nearest_match(&matches, state.top_line);
        state.search_query = query;
        state.search_matches = matches;
        state.current_match = nearest;
    }
}

fn cancel_search(state: &mut PagerState) {
    state.search_input.clear();
    state.search_cursor = 0;
    state.mode = Mode::Normal;
}

fn handle_search_key(state: &mut PagerState, key: &Key) {
    match key {
        Key::Char(c) => {
            let cursor = clamp_cursor_to_boundary(&state.search_input, state.search_cursor);
            state.search_input.insert(cursor, *c);
            state.search_cursor = cursor + c.len_utf8();
        }
        Key::Backspace => {
            let cursor = clamp_cursor_to_boundary(&state.search_input, state.search_cursor);
            if cursor > 0 {
                let remove_at = prev_char_boundary(&state.search_input, cursor);
                state.search_input.remove(remove_at);
                state.search_cursor = remove_at;
            } else {
                state.search_cursor = cursor;
            }
            if state.search_input.is_empty() {
                state.mode = Mode::Normal;
            }
        }
        Key::AltBackspace => {
            let cursor = clamp_cursor_to_boundary(&state.search_input, state.search_cursor);
            let new_pos = clamp_cursor_to_boundary(
                &state.search_input,
                word_boundary_left(&state.search_input, cursor),
            );
            state.search_input.replace_range(new_pos..cursor, "");
            state.search_cursor = new_pos;
            if state.search_input.is_empty() {
                state.mode = Mode::Normal;
            }
        }
        Key::CtrlU => {
            let cursor = clamp_cursor_to_boundary(&state.search_input, state.search_cursor);
            if cursor > 0 {
                state.search_input = state.search_input[cursor..].to_string();
                state.search_cursor = 0;
            } else {
                state.search_cursor = cursor;
            }
            if state.search_input.is_empty() {
                state.mode = Mode::Normal;
            }
        }
        Key::Left => {
            let cursor = clamp_cursor_to_boundary(&state.search_input, state.search_cursor);
            state.search_cursor = prev_char_boundary(&state.search_input, cursor);
        }
        Key::Right => {
            let cursor = clamp_cursor_to_boundary(&state.search_input, state.search_cursor);
            state.search_cursor = next_char_boundary(&state.search_input, cursor);
        }
        Key::AltLeft => {
            let cursor = clamp_cursor_to_boundary(&state.search_input, state.search_cursor);
            state.search_cursor = clamp_cursor_to_boundary(
                &state.search_input,
                word_boundary_left(&state.search_input, cursor),
            );
        }
        Key::AltRight => {
            let cursor = clamp_cursor_to_boundary(&state.search_input, state.search_cursor);
            state.search_cursor = clamp_cursor_to_boundary(
                &state.search_input,
                word_boundary_right(&state.search_input, cursor),
            );
        }
        _ => {}
    }
}

fn open_in_editor(path: &str, line: Option<u32>) {
    tui::pager::open_in_editor(path, line, false);
}

fn resolve_path_for_editor(path: &str, repo: &std::path::Path) -> std::path::PathBuf {
    let file = std::path::Path::new(path);
    if file.is_absolute() {
        file.to_path_buf()
    } else {
        repo.join(file)
    }
}

fn bar_visible(state: &PagerState) -> bool {
    matches!(state.mode, Mode::Search | Mode::Help | Mode::Visual) || !state.status_message.is_empty()
}

fn content_height(rows: u16, state: &PagerState) -> usize {
    if bar_visible(state) {
        rows.saturating_sub(1) as usize
    } else {
        rows as usize
    }
}

fn format_help_lines(cols: usize, content_height: usize) -> Vec<String> {
    let help = keymap_help_lines();

    let mut lines = Vec::with_capacity(content_height);
    let top_pad = content_height.saturating_sub(help.len()) / 2;
    for _ in 0..top_pad {
        lines.push(" ".repeat(cols));
    }

    let max_w = help.iter().map(|h| h.chars().count()).max().unwrap_or(0);
    let left_pad = cols.saturating_sub(max_w) / 2;

    for h in &help {
        if lines.len() >= content_height {
            break;
        }
        let vis_len = h.chars().count();
        if vis_len >= cols {
            lines.push(h.chars().take(cols).collect());
        } else {
            let right_pad = cols.saturating_sub(left_pad + vis_len);
            lines.push(format!(
                "{}{}{}",
                " ".repeat(left_pad),
                h,
                " ".repeat(right_pad)
            ));
        }
    }

    while lines.len() < content_height {
        lines.push(" ".repeat(cols));
    }

    lines
}

fn format_status_bar(state: &PagerState, content_height: usize, cols: usize) -> String {
    if state.mode == Mode::Visual {
        let lo = state.visual_anchor.min(state.cursor_line);
        let hi = state.visual_anchor.max(state.cursor_line);
        let count = hi - lo + 1;
        let left = format!("-- VISUAL -- ({count} lines)");
        let left_len = left.len();
        let pad = cols.saturating_sub(left_len);
        return format!("{left}{}", " ".repeat(pad));
    }

    if state.mode == Mode::Help {
        let left = "? to close";
        let line_count = state.doc.line_count();
        let max_top = max_scroll(line_count, content_height);
        let top = state.top_line.min(max_top);
        let end = (top + content_height).min(line_count);
        let range = format!("{}-{}/{}", top + 1, end, line_count);
        let position = if top == 0 {
            "TOP".to_string()
        } else if end >= line_count {
            "END".to_string()
        } else {
            format!("{}%", (end as f64 / line_count as f64 * 100.0).round() as usize)
        };
        let right = format!("{}{}{} {}", style::DIM, range, style::NO_DIM, position);
        let right_vis = range.len() + 1 + position.len();
        let total_vis = left.len() + right_vis;
        if total_vis >= cols {
            let pad = " ".repeat(cols.saturating_sub(right_vis));
            return format!("{pad}{right}");
        }
        let gap = cols - total_vis;
        return format!("{left}{}{right}", " ".repeat(gap));
    }

    if state.mode == Mode::Search {
        let cursor = clamp_cursor_to_boundary(&state.search_input, state.search_cursor);
        let before = &state.search_input[..cursor];
        let after = &state.search_input[cursor..];
        let cursor_char = if cursor < state.search_input.len() {
            let c = after.chars().next().unwrap();
            let rest = &after[c.len_utf8()..];
            format!("{}{c}{}{}{rest}", style::RESET, style::STATUS_BG, style::STATUS_FG)
        } else {
            format!("{}\u{2588}{}{}", style::RESET, style::STATUS_BG, style::STATUS_FG)
        };
        let content = format!("/{before}{cursor_char}");
        let vis_len = if cursor < state.search_input.len() {
            1 + state.search_input.chars().count()
        } else {
            1 + state.search_input.chars().count() + 1
        };
        let pad = " ".repeat(cols.saturating_sub(vis_len));
        return format!("{content}{pad}");
    }

    if !state.status_message.is_empty() {
        let msg = &state.status_message;
        let pad = " ".repeat(cols.saturating_sub(msg.len()));
        return format!("{msg}{pad}");
    }

    // Normal mode (unreachable when bar_visible is false, but keeps function correct)
    " ".repeat(cols)
}

fn render_content_area(out: &mut impl Write, state: &PagerState, cols: u16, content_rows: u16) {
    let content_height = content_rows as usize;
    let (vis_start, vis_end, _, max_top) = viewport_bounds(state, content_height);
    let top = state.top_line.clamp(vis_start, max_top);

    if state.mode == Mode::Help {
        let help_lines = format_help_lines(cols as usize, content_height);
        for (i, line) in help_lines.iter().enumerate() {
            move_to(out, i as u16, 0);
            let _ = write!(out, "{CLEAR_LINE}{}{line}{}", style::DIM, style::NO_DIM);
        }
        return;
    }

    let diff_w = diff_area_width(cols, state.tree_width, state.tree_visible, state.full_context);
    let show_scrollbar = state.full_context;

    for row in 0..content_height {
        move_to(out, row as u16, 0);
        let idx = top + row;
        if idx >= vis_start && idx < vis_end {
            let mut line = state.doc.lines[idx].clone();
            if !state.search_query.is_empty() {
                line = highlight_search(&line, &state.search_query);
            }
            if state.mode == Mode::Visual {
                let lo = state.visual_anchor.min(state.cursor_line);
                let hi = state.visual_anchor.max(state.cursor_line);
                if idx >= lo && idx <= hi {
                    line = highlight_visual_line(&line, diff_w);
                }
            }
            if idx == state.cursor_line && state.mode != Mode::Visual {
                let vis_w = crate::ansi::visible_width(&line);
                let pad = diff_w.saturating_sub(vis_w);
                line = format!("{}{line}{}{}", style::BG_CURSOR, " ".repeat(pad), style::RESET);
            }
            let _ = write!(out, "{CLEAR_LINE}{line}");
        } else {
            let _ = write!(out, "{CLEAR_LINE}");
        }

        if show_scrollbar {
            let cell = render_scrollbar_cell(
                row,
                content_height,
                vis_start,
                vis_end,
                top,
                &state.doc.line_map,
            );
            let _ = write!(out, "{}\x1b[{}G{cell}", style::RESET, diff_w + 1);
        }

        if state.tree_visible {
            let tree_col = diff_w + 1 + usize::from(show_scrollbar);
            let sep_color = if state.tree_focused() { style::FG_SEP_ACTIVE } else { style::FG_SEP };
            let _ = write!(
                out,
                "{}\x1b[{}G\x1b[K{sep_color}│{}",
                style::RESET,
                tree_col,
                style::RESET,
            );
            if let Some(tree_line) = state.tree_lines.get(state.tree_scroll + row) {
                let _ = write!(out, "{tree_line}");
            }
            let _ = write!(out, "{}", style::RESET);
        }
    }
}

fn render_status_bar(out: &mut impl Write, state: &PagerState, cols: u16, content_rows: u16) {
    let content_height = content_rows as usize;
    move_to(out, content_rows, 0);
    let _ = write!(out, "{CLEAR_LINE}");
    let status = format_status_bar(state, content_height, cols as usize);
    let _ = write!(out, "{}{}{}{}{}", style::RESET, style::STATUS_BG, style::STATUS_FG, status, style::RESET);
}

fn render_screen(out: &mut impl Write, state: &PagerState, cols: u16, rows: u16) {
    let ch = content_height(rows, state);
    render_content_area(out, state, cols, u16::try_from(ch).unwrap_or(u16::MAX));
    if bar_visible(state) {
        render_status_bar(out, state, cols, u16::try_from(ch).unwrap_or(u16::MAX));
    } else {
        // Clear stale status bar content on the last row
        move_to(out, rows - 1, 0);
        let _ = write!(out, "{CLEAR_LINE}");
    }
    let _ = out.flush();
}

fn scroll_to_match(state: &mut PagerState, rows: u16) {
    let Ok(match_idx) = usize::try_from(state.current_match) else {
        return;
    };
    if match_idx >= state.search_matches.len() {
        return;
    }
    let match_line = state.search_matches[match_idx];
    let ch = content_height(rows, state);
    state.cursor_line = match_line;
    enforce_scrolloff(state, ch);
}

/// Jump to next entry in `targets` after current top_line.
pub(crate) fn jump_next(targets: &[usize], top_line: usize) -> Option<usize> {
    targets.iter().find(|&&t| t > top_line).copied()
}

/// Jump to previous entry in `targets` before current top_line.
pub(crate) fn jump_prev(targets: &[usize], top_line: usize) -> Option<usize> {
    targets.iter().rev().find(|&&t| t < top_line).copied()
}

/// Filter targets (hunk_starts or file_starts) to those within `[range_start, range_end)`.
fn targets_in_range(targets: &[usize], range_start: usize, range_end: usize) -> Vec<usize> {
    targets
        .iter()
        .filter(|&&s| s >= range_start && s < range_end)
        .copied()
        .collect()
}

/// Compute change-group starts: lines where Added/Deleted begins after a non-change line.
/// Used for d/u navigation in full-context mode where hunk_starts has only one entry per file.
fn change_group_starts(line_map: &[LineInfo], range_start: usize, range_end: usize) -> Vec<usize> {
    let end = range_end.min(line_map.len());
    let mut starts = Vec::new();
    for i in range_start..end {
        let is_change = matches!(line_map[i].line_kind, Some(LineKind::Added | LineKind::Deleted));
        let prev_is_change = i > range_start
            && matches!(line_map[i - 1].line_kind, Some(LineKind::Added | LineKind::Deleted));
        if is_change && !prev_is_change {
            starts.push(i);
        }
    }
    starts
}

/// Compute one navigation target per hunk by finding the first change line
/// (Added/Deleted) within each hunk span. Falls back to the raw hunk start
/// when a hunk has no change lines.
fn hunk_change_starts(line_map: &[LineInfo], hunk_starts: &[usize]) -> Vec<usize> {
    if hunk_starts.is_empty() {
        return Vec::new();
    }

    let mut targets = Vec::with_capacity(hunk_starts.len());
    for (idx, &start) in hunk_starts.iter().enumerate() {
        let end = hunk_starts
            .get(idx + 1)
            .copied()
            .unwrap_or(line_map.len())
            .min(line_map.len());

        let target = (start..end)
            .find(|&i| matches!(line_map[i].line_kind, Some(LineKind::Added | LineKind::Deleted)))
            .unwrap_or(start);
        targets.push(target);
    }
    targets
}

fn du_nav_targets(state: &PagerState) -> Vec<usize> {
    if state.full_context {
        change_group_starts(&state.doc.line_map, 0, state.doc.line_map.len())
    } else {
        hunk_change_starts(&state.doc.line_map, &state.doc.hunk_starts)
    }
}

/// Result of d/u navigation. Caller must sync_active_file_to_cursor before computing top_line.
struct NavDuResult {
    cursor_line: usize,
    status_message: String,
    moved: bool,
}

/// Shared d/u navigation: next hunk or change group. Caller applies cursor, syncs active_file, then sets top_line.
fn nav_du_down(state: &PagerState, _content_height: usize) -> NavDuResult {
    if state.doc.line_map.is_empty() {
        return NavDuResult {
            cursor_line: state.cursor_line,
            status_message: String::new(),
            moved: false,
        };
    }
    let anchor = state.cursor_line;
    let max_line = state.doc.line_map.len().saturating_sub(1);
    let hunks = du_nav_targets(state);
    if let Some(target) = jump_next(&hunks, anchor) {
        let cursor = next_content_line(&state.doc.line_map, target, max_line);
        let status = nav_status_message(
            if state.full_context { "Change" } else { "Hunk" },
            cursor,
            &hunks,
            &state.doc.line_map,
        );
        NavDuResult { cursor_line: cursor, status_message: status, moved: true }
    } else {
        NavDuResult {
            cursor_line: state.cursor_line,
            status_message: String::new(),
            moved: false,
        }
    }
}

/// Shared u navigation: previous hunk or change group. Caller applies cursor, syncs active_file, then sets top_line.
fn nav_du_up(state: &PagerState, _content_height: usize) -> NavDuResult {
    if state.doc.line_map.is_empty() {
        return NavDuResult {
            cursor_line: state.cursor_line,
            status_message: String::new(),
            moved: false,
        };
    }
    let anchor = state.cursor_line;
    let max_line = state.doc.line_map.len().saturating_sub(1);
    let hunks = du_nav_targets(state);
    if let Some(target) = jump_prev(&hunks, anchor) {
        let mut cursor = next_content_line(&state.doc.line_map, target, max_line);
        if cursor >= anchor {
            if let Some(target2) = jump_prev(&hunks, target) {
                cursor = next_content_line(&state.doc.line_map, target2, max_line);
            } else {
                cursor = anchor;
            }
        }
        let status = nav_status_message(
            if state.full_context { "Change" } else { "Hunk" },
            cursor,
            &hunks,
            &state.doc.line_map,
        );
        NavDuResult { cursor_line: cursor, status_message: status, moved: true }
    } else {
        NavDuResult {
            cursor_line: state.cursor_line,
            status_message: String::new(),
            moved: false,
        }
    }
}

/// Result of D/U navigation: new cursor/top and status.
#[allow(dead_code)]
struct NavDUResult {
    cursor_line: usize,
    top_line: usize,
    status_message: String,
    moved: bool,
}

/// Shared D navigation: next file in visible range.
#[allow(non_snake_case)]
fn nav_D_down(state: &PagerState, content_height: usize) -> NavDUResult {
    let anchor = state.cursor_line;
    let (rs, re) = visible_range(state);
    let max_line = re.saturating_sub(1);
    let max_top = re.saturating_sub(content_height).max(rs);
    let files = targets_in_range(&state.doc.file_starts, rs, re);
    if let Some(target) = jump_next(&files, anchor) {
        let cursor = next_content_line(&state.doc.line_map, target, max_line);
        let top = cursor
            .saturating_sub(content_height / 2)
            .max(rs)
            .min(max_top);
        let status = file_status_message(cursor, &files, &state.doc.line_map);
        NavDUResult {
            cursor_line: cursor,
            top_line: top,
            status_message: status,
            moved: true,
        }
    } else {
        NavDUResult {
            cursor_line: state.cursor_line,
            top_line: state.top_line,
            status_message: String::new(),
            moved: false,
        }
    }
}

/// Shared U navigation: previous file in visible range.
#[allow(non_snake_case)]
fn nav_U_up(state: &PagerState, content_height: usize) -> NavDUResult {
    let anchor = state.cursor_line;
    let (rs, re) = visible_range(state);
    let max_line = re.saturating_sub(1);
    let max_top = re.saturating_sub(content_height).max(rs);
    let files = targets_in_range(&state.doc.file_starts, rs, re);
    if let Some(target) = jump_prev(&files, anchor) {
        let mut cursor = next_content_line(&state.doc.line_map, target, max_line);
        if cursor >= anchor {
            if let Some(target2) = jump_prev(&files, target) {
                cursor = next_content_line(&state.doc.line_map, target2, max_line);
            } else {
                cursor = anchor;
            }
        }
        let top = cursor
            .saturating_sub(content_height / 2)
            .max(rs)
            .min(max_top);
        let status = file_status_message(cursor, &files, &state.doc.line_map);
        NavDUResult {
            cursor_line: cursor,
            top_line: top,
            status_message: status,
            moved: true,
        }
    } else {
        NavDUResult {
            cursor_line: state.cursor_line,
            top_line: state.top_line,
            status_message: String::new(),
            moved: false,
        }
    }
}

/// Build a status message like "Hunk 3/7 · b.rs" (or "Change 3/7 · b.rs" in
/// full-context mode) for the navigation target containing `cursor`.
fn nav_status_message(label: &str, cursor: usize, starts: &[usize], line_map: &[LineInfo]) -> String {
    if starts.is_empty() {
        return String::new();
    }
    let idx = starts.partition_point(|&s| s <= cursor).saturating_sub(1);
    let path = line_map.get(cursor).map_or("", |li| li.path.as_str());
    format!("{label} {}/{} \u{00b7} {}", idx + 1, starts.len(), path)
}

/// Build a status message like "File 2/5 · src/main.rs" for file navigation.
fn file_status_message(cursor: usize, file_starts: &[usize], line_map: &[LineInfo]) -> String {
    nav_status_message("File", cursor, file_starts, line_map)
}

/// Builds a JSON object string of common reducer/document-swap instrumentation fields.
/// Used only when GD_DEBUG=1; does not mutate state.
fn format_debug_state(state: &PagerState) -> String {
    let (rs, re) = visible_range(state);
    let active_file_valid = state.active_file().is_none_or(|idx| idx < state.doc.file_starts.len());
    let tree_cursor_file_idx = state
        .tree_entries
        .get(state.tree_cursor())
        .and_then(|e| e.file_idx)
        .map_or(String::from("null"), |v| v.to_string());
    format!(
        "{{\"treeVisible\":{},\"treeFocused\":{},\"activeFile\":{},\"activeFileValid\":{},\"fullContext\":{},\"cursorLine\":{},\"topLine\":{},\"rangeStart\":{},\"rangeEnd\":{},\"lineMapLen\":{},\"fileStartsLen\":{},\"treeCursor\":{},\"treeCursorFileIdx\":{}}}",
        state.tree_visible,
        state.tree_focused(),
        state.active_file().map_or(String::from("null"), |v| v.to_string()),
        active_file_valid,
        state.full_context,
        state.cursor_line,
        state.top_line,
        rs,
        re,
        state.doc.line_map.len(),
        state.doc.file_starts.len(),
        state.tree_cursor(),
        tree_cursor_file_idx
    )
}

/// Re-render the diff at a new width. Captures anchor, builds fresh Document, swaps, and remaps.
fn re_render(
    state: &mut PagerState,
    files: &[DiffFile],
    color: bool,
    cols: u16,
) {
    let anchor = capture_view_anchor(state);
    let width = diff_area_width(cols, state.tree_width, state.tree_visible, state.full_context);
    let output = render::render(files, width, color, state.tree_visible);
    let new_doc = Document::from_render_output(output);
    remap_after_document_swap(state, anchor, new_doc, files);

    debug_trace("pager.rs:re_render", "post rerender state", &format_debug_state(state));
}

#[derive(Debug)]
pub(crate) struct TreeEntry {
    pub(crate) label: String,
    pub(crate) depth: usize,
    pub(crate) file_idx: Option<usize>,
    pub(crate) status: Option<FileStatus>,
    pub(crate) collapsed: bool,
}

pub(crate) fn build_tree_entries(files: &[DiffFile]) -> Vec<TreeEntry> {
    // Sort files by path for grouping
    let mut indexed: Vec<(usize, &str)> = files.iter().enumerate().map(|(i, f)| (i, f.path())).collect();
    indexed.sort_by(|a, b| a.1.cmp(b.1));

    let mut entries: Vec<TreeEntry> = Vec::new();
    let mut prev_components: Vec<&str> = Vec::new();

    for (file_idx, path) in &indexed {
        let parts: Vec<&str> = path.split('/').collect();
        let dir_parts = &parts[..parts.len() - 1];
        let basename = parts[parts.len() - 1];

        // Find common prefix length with previous path's directory components
        let common = prev_components
            .iter()
            .zip(dir_parts.iter())
            .take_while(|(a, b)| a == b)
            .count();

        // Emit new directory entries for components beyond the common prefix
        for (depth, &component) in dir_parts.iter().enumerate().skip(common) {
            entries.push(TreeEntry {
                label: component.to_string(),
                depth,
                file_idx: None,
                status: None,
                collapsed: false,
            });
        }

        // Emit the file entry
        entries.push(TreeEntry {
            label: basename.to_string(),
            depth: dir_parts.len(),
            file_idx: Some(*file_idx),
            status: Some(files[*file_idx].status),
            collapsed: false,
        });

        prev_components = dir_parts.to_vec();
    }

    // Post-process: collapse single-child directories
    let mut i = 0;
    while i + 1 < entries.len() {
        if entries[i].file_idx.is_none() && entries[i + 1].file_idx.is_none()
            && entries[i + 1].depth == entries[i].depth + 1
        {
            // Check that the parent has no other children (no sibling at same depth between i+1 and next entry at parent depth or lower)
            let parent_depth = entries[i].depth;
            let has_sibling = entries[i + 2..].iter().any(|e| e.depth <= parent_depth + 1 && e.depth > parent_depth)
                && entries[i + 2..].iter().take_while(|e| e.depth > parent_depth).any(|e| e.depth == parent_depth + 1);
            if !has_sibling {
                // Merge: join labels with '/'
                let child_label = entries[i + 1].label.clone();
                entries[i].label = format!("{}/{}", entries[i].label, child_label);
                // All subsequent entries that were children of the removed child dir need depth decremented
                let removed_depth = entries[i + 1].depth;
                entries.remove(i + 1);
                for e in &mut entries[i + 1..] {
                    if e.depth > removed_depth {
                        e.depth -= 1;
                    } else {
                        break;
                    }
                }
                continue; // Re-check in case of further collapsing
            }
        }
        i += 1;
    }

    entries
}

pub(crate) fn compute_tree_width(tree_entries: &[TreeEntry]) -> usize {
    let max_len = tree_entries
        .iter()
        .map(|e| {
            // connectors: (depth+1)*4
            // icon+space: 2
            // status symbol (file entries with status only): +2 for "X " char+space
            // label + padding: label.len() + 2
            let status_extra = if e.file_idx.is_some() && e.status.is_some() { 2 } else { 0 };
            (e.depth + 1) * 4 + 2 + status_extra + e.label.len() + 2
        })
        .max()
        .unwrap_or(0);
    max_len.min(40)
}

fn file_idx_to_entry_idx(tree_entries: &[TreeEntry], file_idx: usize) -> usize {
    tree_entries
        .iter()
        .position(|e| e.file_idx == Some(file_idx))
        .unwrap_or(0)
}

/// Build the Unicode box-drawing prefix for a visible entry at `idx`.
/// Each depth level contributes 4 characters: either a continuation pipe
/// (`│   `) or blank (`    `), and the entry's own connector (`├── ` or `└── `).
/// Operates on the filtered visible slice so connectors reflect hidden entries.
fn compute_connector_prefix(visible: &[&TreeEntry], idx: usize) -> String {
    let depth = visible[idx].depth;
    let mut prefix = String::new();

    // Ancestor columns: for each depth 0..depth-1, draw a continuation pipe
    // if any subsequent visible entry returns to that depth
    for d in 0..depth {
        let has_continuation = visible[idx + 1..].iter().any(|e| e.depth <= d);
        if has_continuation {
            prefix.push_str("│   ");
        } else {
            prefix.push_str("    ");
        }
    }

    // Entry's own connector: check if a visible sibling follows at the same depth
    let has_sibling_after = visible[idx + 1..]
        .iter()
        .take_while(|e| e.depth >= depth)
        .any(|e| e.depth == depth);
    if has_sibling_after {
        prefix.push_str("├── ");
    } else {
        prefix.push_str("└── ");
    }

    prefix
}

/// Return `(symbol_str, ansi_color)` for a file's change status.
fn status_symbol(status: FileStatus) -> (&'static str, &'static str) {
    match status {
        FileStatus::Modified => ("M", style::FG_STATUS_MODIFIED),
        FileStatus::Added => ("A", style::FG_STATUS_ADDED),
        FileStatus::Deleted => ("D", style::FG_STATUS_DELETED),
        FileStatus::Renamed => ("R", style::FG_STATUS_RENAMED),
        FileStatus::Untracked => ("?", style::FG_STATUS_UNTRACKED),
    }
}

/// Build display lines for visible tree entries, returning both the rendered
/// lines and a mapping from visible-line index to original `tree_entries` index.
fn build_tree_lines(
    tree_entries: &[TreeEntry],
    cursor_entry_idx: usize,
    width: usize,
) -> (Vec<String>, Vec<usize>) {
    // Build visibility filter: an entry is hidden if any ancestor dir is collapsed
    let mut visible: Vec<&TreeEntry> = Vec::new();
    let mut visible_orig: Vec<usize> = Vec::new();
    let mut collapse_depth: Option<usize> = None; // depth at which a collapse is active

    for (i, entry) in tree_entries.iter().enumerate() {
        if let Some(cd) = collapse_depth {
            if entry.depth > cd {
                continue; // hidden under a collapsed dir
            }
            collapse_depth = None; // back to or above collapsed dir's level
        }
        visible.push(entry);
        visible_orig.push(i);
        if entry.file_idx.is_none() && entry.collapsed {
            collapse_depth = Some(entry.depth);
        }
    }

    let mut lines = Vec::new();

    for (vi, &entry) in visible.iter().enumerate() {
        let orig_idx = visible_orig[vi];
        let prefix = compute_connector_prefix(&visible, vi);
        let (icon, icon_color) = if entry.file_idx.is_some() {
            style::file_icon(&entry.label)
        } else {
            style::dir_icon(entry.collapsed)
        };

        // prefix is (depth+1)*4 chars, plus icon(1) + space(1) + [status(1) + space(1) when present] + label
        let status_extra = if entry.file_idx.is_some() && entry.status.is_some() { 2 } else { 0 };
        let vis_len = (entry.depth + 1) * 4 + 2 + status_extra + entry.label.chars().count();
        let right_pad = width.saturating_sub(vis_len);
        let guide = style::FG_TREE_GUIDE;

        if orig_idx == cursor_entry_idx {
            let reset = style::RESET;
            let fg = style::FG_FILE_HEADER;
            let bg = style::BG_TREE_CURSOR;
            let label = &entry.label;
            let rpad = " ".repeat(right_pad);
            if entry.file_idx.is_some() {
                if let Some(st) = entry.status {
                    let (sc, sc_color) = status_symbol(st);
                    lines.push(format!("{bg}{guide}{prefix}{reset}{bg}{icon_color}{icon} {sc_color}{sc}{fg} {label}{rpad}{reset}"));
                } else {
                    lines.push(format!("{bg}{guide}{prefix}{reset}{bg}{icon_color}{icon} {fg}{label}{rpad}{reset}"));
                }
            } else {
                lines.push(format!("{bg}{guide}{prefix}{reset}{bg}{icon_color}{icon} {fg}{label}{rpad}{reset}"));
            }
        } else if entry.file_idx.is_some() {
            let reset = style::RESET;
            let fg = style::FG_TREE;
            let label = &entry.label;
            let rpad = " ".repeat(right_pad);
            if let Some(st) = entry.status {
                let (sc, sc_color) = status_symbol(st);
                lines.push(format!("{guide}{prefix}{reset}{icon_color}{icon}{reset} {sc_color}{sc}{reset} {fg}{label}{rpad}{reset}"));
            } else {
                lines.push(format!("{guide}{prefix}{reset}{icon_color}{icon}{reset} {fg}{label}{rpad}{reset}"));
            }
        } else {
            let reset = style::RESET;
            let fg = style::FG_TREE_DIR;
            let label = &entry.label;
            let rpad = " ".repeat(right_pad);
            lines.push(format!("{guide}{prefix}{reset}{icon_color}{icon}{reset} {fg}{label}{rpad}{reset}"));
        }
    }

    (lines, visible_orig)
}

fn sync_tree_cursor(state: &mut PagerState, content_height: usize) {
    if !state.tree_visible || state.tree_focused() {
        return;
    }
    sync_tree_cursor_force(state, content_height);
}

fn sync_tree_cursor_force(state: &mut PagerState, content_height: usize) {
    if !state.tree_visible {
        return;
    }
    if state.tree_entries.is_empty() {
        return;
    }
    let new_cursor = if let Some(fi) = state.active_file() {
        fi
    } else {
        state
            .doc.line_map
            .get(state.cursor_line)
            .map_or(0, |li| li.file_idx)
    };
    let mut new_entry_idx = file_idx_to_entry_idx(&state.tree_entries, new_cursor);
    // If the target entry is hidden (collapsed parent), find nearest visible ancestor
    if !state.tree_visible_to_entry.contains(&new_entry_idx) {
        let target_depth = state
            .tree_entry(new_entry_idx)
            .map(|e| e.depth)
            .unwrap_or(0);
        new_entry_idx = state.tree_entries[..new_entry_idx]
            .iter()
            .rposition(|e| e.file_idx.is_none() && e.depth < target_depth)
            .unwrap_or(0);
    }
    if new_entry_idx != state.tree_cursor() {
        state.set_tree_cursor(new_entry_idx);
        let (tl, tv) = build_tree_lines(&state.tree_entries, state.tree_cursor(), state.tree_width);
        state.tree_lines = tl;
        state.tree_visible_to_entry = tv;
        ensure_tree_cursor_visible(state, content_height);
    }
}

fn sync_active_file_to_cursor(state: &mut PagerState) {
    if state.active_file().is_none() {
        return;
    }
    if let Some(file_idx) = state.doc.line_map.get(state.cursor_line).map(|li| li.file_idx) {
        state.set_active_file(Some(file_idx));
    }
}

fn ensure_tree_cursor_visible(state: &mut PagerState, content_height: usize) {
    // Translate entry index to visible-line offset
    let offset = state
        .tree_visible_to_entry
        .iter()
        .position(|&ei| ei == state.tree_cursor())
        .unwrap_or(0);
    if offset < state.tree_scroll {
        state.tree_scroll = offset;
    }
    if offset >= state.tree_scroll + content_height {
        state.tree_scroll = offset + 1 - content_height;
    }
}

fn move_tree_selection(state: &mut PagerState, delta: isize, content_height: usize) -> bool {
    let Some(current_visible_idx) = state
        .tree_visible_to_entry
        .iter()
        .position(|&entry_idx| entry_idx == state.tree_cursor())
    else {
        return false;
    };
    let next_visible_idx = current_visible_idx as isize + delta;
    if next_visible_idx < 0 || next_visible_idx >= state.tree_visible_to_entry.len() as isize {
        return false;
    }
    state.set_tree_cursor(state.tree_visible_to_entry[next_visible_idx as usize]);
    let (tree_lines, tree_visible_to_entry) =
        build_tree_lines(&state.tree_entries, state.tree_cursor(), state.tree_width);
    state.tree_lines = tree_lines;
    state.tree_visible_to_entry = tree_visible_to_entry;
    ensure_tree_cursor_visible(state, content_height);
    true
}

// -------- Reducer and mode-specific handlers (chunk-02) --------

fn dispatch_search_action(state: &mut PagerState, action: ActionId) -> ReducerEffect {
    match action {
        ActionId::SearchSubmit => {
            submit_search(state);
            ReducerEffect::Continue
        }
        ActionId::SearchCancel => {
            cancel_search(state);
            ReducerEffect::Continue
        }
        _ => ReducerEffect::Continue,
    }
}

fn reduce_search(state: &mut PagerState, event: &ReducerEvent, ctx: &ReducerCtx<'_>) -> ReducerEffect {
    let ReducerEvent::Key(key) = event;
    if let Some(action) = keymap_lookup(*key, KeyContext::Search) {
        return dispatch_search_action(state, action);
    }
    handle_search_key(state, key);
    if state.mode == Mode::Normal && state.current_match >= 0 {
        scroll_to_match(state, ctx.rows);
        sync_tree_cursor(state, ctx.content_height);
    }
    ReducerEffect::Continue
}

fn reduce_help(state: &mut PagerState, _event: &ReducerEvent, _ctx: &ReducerCtx<'_>) -> ReducerEffect {
    state.mode = Mode::Normal;
    ReducerEffect::Continue
}

fn reduce_visual(state: &mut PagerState, event: &ReducerEvent, ctx: &ReducerCtx<'_>) -> ReducerEffect {
    let ReducerEvent::Key(key) = event;
    if let Some(action) = keymap_lookup(*key, KeyContext::Visual) {
        return dispatch_visual_action(state, action, ctx);
    }
    ReducerEffect::Continue
}

fn reduce_tree(state: &mut PagerState, event: &ReducerEvent, ctx: &ReducerCtx<'_>) -> ReducerEffect {
    let ReducerEvent::Key(key) = event;
    if let Some(action) = keymap_lookup(*key, KeyContext::Tree) {
        return dispatch_tree_action(state, action, ctx);
    }
    reduce_normal(state, event, ctx)
}

fn reduce_normal(
    state: &mut PagerState,
    event: &ReducerEvent,
    ctx: &ReducerCtx<'_>,
) -> ReducerEffect {
    let ReducerEvent::Key(key) = event;
    let ch = ctx.content_height;

    state.status_message.clear();

    if let Some(action) = keymap_lookup(*key, KeyContext::Normal) {
        if let Some(effect) = dispatch_normal_action(state, action, ctx) {
            return effect;
        }
    }

    enforce_scrolloff(state, ch);
    sync_tree_cursor(state, ch);
    ReducerEffect::Continue
}

fn reduce(state: &mut PagerState, event: ReducerEvent, ctx: &ReducerCtx<'_>) -> ReducerEffect {
    let effect = if state.mode == Mode::Search {
        reduce_search(state, &event, ctx)
    } else if state.mode == Mode::Help {
        reduce_help(state, &event, ctx)
    } else if state.mode == Mode::Visual {
        reduce_visual(state, &event, ctx)
    } else if state.tree_focused() {
        reduce_tree(state, &event, ctx)
    } else {
        reduce_normal(state, &event, ctx)
    };
    clamp_cursor_and_top(state);
    debug_assert_valid_state(state);
    effect
}

/// Thin adapter: map Key to ReducerEvent, run reducer, return KeyResult.
pub(crate) fn handle_key(
    state: &mut PagerState,
    key: Key,
    ch: usize,
    rows: u16,
    files: &[DiffFile],
) -> KeyResult {
    let event = ReducerEvent::Key(key);
    let ctx = ReducerCtx {
        content_height: ch,
        rows,
        files,
    };
    KeyResult::from(reduce(state, event, &ctx))
}

fn regenerate_files(diff_ctx: &DiffContext, full_context: bool) -> Vec<DiffFile> {
    let diff_args = if full_context {
        diff_ctx.source.diff_args_full_context()
    } else {
        diff_ctx.source.diff_args()
    };
    let str_args: Vec<&str> = diff_args.iter().map(String::as_str).collect();
    let raw = crate::git::run_diff(&diff_ctx.repo, &str_args);
    let mut files = crate::git::diff::parse(&raw);

    if !diff_ctx.no_untracked && matches!(diff_ctx.source, crate::git::DiffSource::WorkingTree) {
        let max_size: u64 = 256 * 1024;
        for path in crate::git::untracked_files(&diff_ctx.repo) {
            let full = diff_ctx.repo.join(&path);
            let Ok(meta) = full.metadata() else {
                continue;
            };
            if !meta.is_file() || meta.len() > max_size {
                continue;
            }
            let Ok(content) = std::fs::read(&full) else {
                continue;
            };
            if content.contains(&0) {
                continue;
            }
            let text = String::from_utf8_lossy(&content);
            files.push(DiffFile::from_content(&path, &text));
        }
    }

    files
}

pub fn run_pager(output: RenderOutput, files: Vec<DiffFile>, color: bool, diff_ctx: &DiffContext) {
    let mut files = files;
    let mut stdout = io::BufWriter::new(io::stdout());

    let _ = write!(stdout, "{ALT_SCREEN_ON}{CURSOR_HIDE}");
    let _ = stdout.flush();
    let _ = crossterm::terminal::enable_raw_mode();

    let tree_entries = build_tree_entries(&files);
    let mut state = PagerState::new(
        output.lines,
        output.line_map,
        output.file_starts,
        output.hunk_starts,
        tree_entries,
    );

    let mut last_size = get_term_size();
    files = regenerate_files(diff_ctx, state.full_context);
    re_render(&mut state, &files, color, last_size.0);
    let ch = content_height(last_size.1, &state);
    ensure_tree_cursor_visible(&mut state, ch);
    render_screen(&mut stdout, &state, last_size.0, last_size.1);

    loop {
        let ev = match event::poll(Duration::from_millis(50)) {
            Ok(true) => match event::read() {
                Ok(ev) => ev,
                Err(_) => break,
            },
            Ok(false) => {
                let current_size = get_term_size();
                if current_size != last_size {
                    last_size = current_size;
                    re_render(&mut state, &files, color, last_size.0);
                    let ch = content_height(last_size.1, &state);
                    ensure_tree_cursor_visible(&mut state, ch);
                    render_screen(&mut stdout, &state, last_size.0, last_size.1);
                }
                continue;
            }
            Err(_) => break,
        };

        let key = match ev {
            Event::Resize(_, _) => {
                last_size = get_term_size();
                re_render(&mut state, &files, color, last_size.0);
                let ch = content_height(last_size.1, &state);
                ensure_tree_cursor_visible(&mut state, ch);
                render_screen(&mut stdout, &state, last_size.0, last_size.1);
                continue;
            }
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                crossterm_to_key(key_event)
            }
            _ => continue,
        };

        let ch = content_height(last_size.1, &state);
        let result = handle_key(&mut state, key, ch, last_size.1, &files);
        match result {
            KeyResult::Quit => break,
            KeyResult::ReRender => {
                re_render(&mut state, &files, color, last_size.0);
            }
            KeyResult::ReGenerate => {
                let base = format_debug_state(&state);
                debug_trace(
                    "pager.rs:run_pager:regenerate:before",
                    "regenerate start",
                    &format!("{},\"filesLen\":{}}}", base.trim_end_matches('}'), files.len()),
                );
                files = regenerate_files(diff_ctx, state.full_context);
                if files.is_empty() {
                    break; // No diff: exit pager cleanly
                }
                re_render(&mut state, &files, color, last_size.0);
                let base = format_debug_state(&state);
                debug_trace(
                    "pager.rs:run_pager:regenerate:after",
                    "regenerate complete",
                    &format!("{},\"filesLen\":{}}}", base.trim_end_matches('}'), files.len()),
                );
            }
            KeyResult::OpenEditor { path, lineno } => {
                let _ = crossterm::terminal::disable_raw_mode();
                let _ = write!(stdout, "{CURSOR_SHOW}{ALT_SCREEN_OFF}");
                let _ = stdout.flush();

                let resolved = resolve_path_for_editor(&path, &diff_ctx.repo);
                open_in_editor(resolved.to_string_lossy().as_ref(), lineno);

                let _ = write!(stdout, "{ALT_SCREEN_ON}{CURSOR_HIDE}");
                let _ = stdout.flush();
                let _ = crossterm::terminal::enable_raw_mode();
                last_size = get_term_size();
            }
            KeyResult::Continue => {}
        }
        render_screen(&mut stdout, &state, last_size.0, last_size.1);
    }

    let _ = crossterm::terminal::disable_raw_mode();
    let _ = write!(stdout, "{CURSOR_SHOW}{ALT_SCREEN_OFF}");
    let _ = stdout.flush();
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_debug_snapshot;
    use insta::assert_snapshot;
    use std::sync::{Mutex, OnceLock};
    use tui::search::{find_matches, find_nearest_match};

    fn gd_debug_env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    struct GdDebugRestore(Option<std::ffi::OsString>);
    #[allow(unsafe_code)]
    impl Drop for GdDebugRestore {
        fn drop(&mut self) {
            // SAFETY: Tests use a global mutex guard to serialize env mutation.
            match self.0.take() {
                Some(v) => unsafe { std::env::set_var("GD_DEBUG", v) },
                None => unsafe { std::env::remove_var("GD_DEBUG") },
            }
        }
    }

    #[allow(unsafe_code)]
    fn with_gd_debug_env<T>(value: Option<&str>, f: impl FnOnce() -> T) -> T {
        let _guard = gd_debug_env_lock()
            .lock()
            .expect("failed to lock GD_DEBUG test mutex");
        let _restore = GdDebugRestore(std::env::var_os("GD_DEBUG"));
        match value {
            Some(v) => unsafe { std::env::set_var("GD_DEBUG", v) },
            None => unsafe { std::env::remove_var("GD_DEBUG") },
        }
        f()
    }

    #[derive(Debug, PartialEq)]
    #[allow(dead_code)]
    struct StateSnapshot {
        cursor_line: usize,
        top_line: usize,
        active_file: Option<usize>,
        tree_visible: bool,
        tree_focused: bool,
        tree_cursor: usize,
        mode: Mode,
        status_message: String,
        full_context: bool,
        visual_anchor: usize,
    }

    impl From<&PagerState> for StateSnapshot {
        fn from(s: &PagerState) -> Self {
            StateSnapshot {
                cursor_line: s.cursor_line,
                top_line: s.top_line,
                active_file: s.active_file(),
                tree_visible: s.tree_visible,
                tree_focused: s.tree_focused(),
                tree_cursor: s.tree_cursor(),
                mode: s.mode.clone(),
                status_message: s.status_message.clone(),
                full_context: s.full_context,
                visual_anchor: s.visual_anchor,
            }
        }
    }

    /// Invariant test harness: validates PagerState after transitions.
    /// Run in tests to catch invalid-state regressions.
    fn assert_state_invariants(state: &PagerState) {
        let line_count = state.doc.line_count();
        let (rs, re) = visible_range(state);
        let range_max = re.saturating_sub(1);

        // cursor and top within visible range
        assert!(
            state.cursor_line >= rs && state.cursor_line <= range_max,
            "cursor_line {} out of visible range [{}, {}]",
            state.cursor_line,
            rs,
            range_max
        );
        let max_top = range_max.saturating_sub(1).min(line_count.saturating_sub(1));
        assert!(
            state.top_line >= rs && state.top_line <= max_top.min(range_max),
            "top_line {} out of range [rs={}, max_top={}]",
            state.top_line,
            rs,
            max_top
        );

        // single-file scope references existing file
        if let ViewScope::SingleFile(ix) = state.view_scope {
            assert!(
                ix.get() < state.file_count(),
                "SingleFile index {} out of bounds (file_count={})",
                ix.get(),
                state.file_count()
            );
        }

        // tree focus implies valid tree selection
        if state.tree_focused() {
            assert!(
                !state.tree_entries.is_empty(),
                "tree focus invalid when tree empty"
            );
            assert!(
                state.tree_selection.is_some(),
                "tree focus requires valid tree_selection"
            );
            if let Some(sel) = state.tree_selection {
                assert!(
                    sel.get() < state.tree_entry_count(),
                    "tree_selection {} out of bounds (entry_count={})",
                    sel.get(),
                    state.tree_entry_count()
                );
            }
        }

        // overlay payloads valid for current doc
        for &idx in &state.search_matches {
            assert!(
                idx < line_count,
                "search_match index {} out of doc bounds (line_count={})",
                idx,
                line_count
            );
        }
        assert!(
            state.visual_anchor < line_count || line_count == 0,
            "visual_anchor {} out of doc bounds (line_count={})",
            state.visual_anchor,
            line_count
        );
    }

    /// Build a 90-line PagerState for keybinding snapshot tests.
    ///
    /// Layout: 3 files (a.rs lines 0-29, b.rs lines 30-59, c.rs lines 60-89).
    /// File headers at 0, 30, 60. Hunk headers at 5, 15, 35, 45, 65, 75.
    /// All other lines are Context content lines.
    /// Tree: 3 flat file entries, visible, not focused.
    fn make_keybinding_state() -> PagerState {
        let mut line_map = Vec::with_capacity(90);
        let header_indices: &[usize] = &[0, 5, 15, 30, 35, 45, 60, 65, 75];
        for i in 0..90 {
            let (file_idx, path) = if i < 30 {
                (0, "a.rs")
            } else if i < 60 {
                (1, "b.rs")
            } else {
                (2, "c.rs")
            };
            let line_kind = if header_indices.contains(&i) {
                None
            } else {
                Some(LineKind::Context)
            };
            line_map.push(LineInfo {
                file_idx,
                path: path.into(),
                new_lineno: Some(i as u32 + 1),
                old_lineno: None,
                line_kind,
            });
        }

        let tree_entries = vec![
            entry("a.rs", 0, Some(0)),
            entry("b.rs", 0, Some(1)),
            entry("c.rs", 0, Some(2)),
        ];
        let mut state = PagerState::new(
            vec!["line".into(); 90],
            line_map,
            vec![0, 30, 60],
            vec![5, 15, 35, 45, 65, 75],
            tree_entries,
        );
        state.cursor_line = 1;
        state.set_tree_focused(false);
        state
    }

    fn make_search_state(input: &str, cursor: usize) -> PagerState {
        let mut state = make_keybinding_state();
        state.mode = Mode::Search;
        state.search_input = input.to_string();
        state.search_cursor = cursor;
        state
    }

    #[test]
    fn test_search_left_right_accented() {
        let mut state = make_search_state("cafe\u{301}", "cafe\u{301}".len());
        handle_search_key(&mut state, &Key::Left);
        assert_eq!(state.search_cursor, 4);
        handle_search_key(&mut state, &Key::Left);
        assert_eq!(state.search_cursor, 3);
        handle_search_key(&mut state, &Key::Right);
        assert_eq!(state.search_cursor, 4);
        handle_search_key(&mut state, &Key::Right);
        assert_eq!(state.search_cursor, 6);
    }

    #[test]
    fn test_search_backspace_accented() {
        let mut state = make_search_state("nai\u{308}ve", 5);
        handle_search_key(&mut state, &Key::Backspace);
        assert_eq!(state.search_input, "naive");
        assert_eq!(state.search_cursor, 3);
    }

    #[test]
    fn test_search_alt_backspace_multibyte() {
        let mut state = make_search_state("hello \u{4e16}\u{754c}", "hello \u{4e16}\u{754c}".len());
        handle_search_key(&mut state, &Key::AltBackspace);
        assert_eq!(state.search_input, "hello ");
        assert_eq!(state.search_cursor, "hello ".len());
    }

    #[test]
    fn test_search_ctrl_u_emoji() {
        let mut state = make_search_state("a\u{1f600}b", "a\u{1f600}b".len());
        handle_search_key(&mut state, &Key::CtrlU);
        assert_eq!(state.search_input, "");
        assert_eq!(state.search_cursor, 0);
        assert_eq!(state.mode, Mode::Normal);
    }

    #[test]
    fn test_format_status_bar_emoji() {
        let state = make_search_state("\u{1f50d}test", 4);
        let status = format_status_bar(&state, 10, 40);
        let stripped = crate::ansi::strip_ansi(&status);
        assert!(stripped.contains("/\u{1f50d}test"));
    }

    #[test]
    fn test_format_status_bar_mid_char_no_panic() {
        let state = make_search_state("a\u{1f50d}", 2);
        let result = std::panic::catch_unwind(|| format_status_bar(&state, 10, 40));
        assert!(result.is_ok());
    }

    #[test]
    fn test_resolve_path_relative_joins_repo_root() {
        let repo = std::path::Path::new("/tmp/my_repo");
        let path = "src/foo.rs";
        assert_eq!(
            resolve_path_for_editor(path, repo),
            std::path::PathBuf::from("/tmp/my_repo/src/foo.rs")
        );
    }

    #[test]
    fn test_resolve_path_absolute_unchanged() {
        let repo = std::path::Path::new("/tmp/my_repo");
        let path = "/absolute/path/to/file.rs";
        assert_eq!(
            resolve_path_for_editor(path, repo),
            std::path::PathBuf::from(path)
        );
    }

    #[test]
    fn test_resolve_path_simple_filename() {
        let repo = std::path::Path::new("/home/user/repo");
        let path = "README.md";
        assert_eq!(
            resolve_path_for_editor(path, repo),
            std::path::PathBuf::from("/home/user/repo/README.md")
        );
    }

    fn entry(label: &str, depth: usize, file_idx: Option<usize>) -> TreeEntry {
        TreeEntry {
            label: label.to_string(),
            depth,
            file_idx,
            status: file_idx.map(|_| FileStatus::Modified),
            collapsed: false,
        }
    }

    #[test]
    fn change_group_starts_finds_change_boundaries() {
        // Simulate a full-context file: Context...Added...Context...Deleted...Context
        let line_map: Vec<LineInfo> = [
            // 0-4: Context
            Some(LineKind::Context), Some(LineKind::Context), Some(LineKind::Context),
            Some(LineKind::Context), Some(LineKind::Context),
            // 5-7: Added group
            Some(LineKind::Added), Some(LineKind::Added), Some(LineKind::Added),
            // 8-11: Context
            Some(LineKind::Context), Some(LineKind::Context),
            Some(LineKind::Context), Some(LineKind::Context),
            // 12-13: Deleted group
            Some(LineKind::Deleted), Some(LineKind::Deleted),
            // 14-16: Context
            Some(LineKind::Context), Some(LineKind::Context), Some(LineKind::Context),
            // 17: Added (single line)
            Some(LineKind::Added),
            // 18-19: Context
            Some(LineKind::Context), Some(LineKind::Context),
        ]
        .iter()
        .enumerate()
        .map(|(i, kind)| LineInfo {
            file_idx: 0,
            path: "test.rs".into(),
            new_lineno: Some(i as u32 + 1),
            old_lineno: None,
            line_kind: *kind,
        })
        .collect();

        let starts = change_group_starts(&line_map, 0, line_map.len());
        assert_eq!(starts, vec![5, 12, 17]);
    }

    #[test]
    fn key_d_full_context_single_file_navigates_changes() {
        let mut state = make_keybinding_state();
        state.set_active_file(Some(0));
        state.full_context = true;
        // Override line_map for file 0 (lines 0-29) to have change groups
        // Keep headers at 0, 5, 15 as None; add changes at 7-8 and 20-21
        for i in 0..30 {
            state.doc.line_map[i].line_kind = if [0, 5, 15].contains(&i) {
                None
            } else if (7..=8).contains(&i) {
                Some(LineKind::Added)
            } else if (20..=21).contains(&i) {
                Some(LineKind::Deleted)
            } else {
                Some(LineKind::Context)
            };
        }
        state.cursor_line = 1; // on a context line
        handle_key(&mut state, Key::Char('d'), 40, 40, &[]);
        // Should jump to first change group at 7
        assert_state_invariants(&state);
        assert_debug_snapshot!(StateSnapshot::from(&state));
    }

    #[test]
    fn nav_status_message_positions() {
        let state = make_keybinding_state();
        // hunk_starts: [5, 15, 35, 45, 65, 75]
        // Cursor on first content line after hunk_start 5 → Hunk 1/6 · a.rs
        assert_eq!(
            nav_status_message("Hunk", 6, &state.doc.hunk_starts, &state.doc.line_map),
            "Hunk 1/6 · a.rs"
        );
        // Cursor on first content line after hunk_start 35 → Hunk 3/6 · b.rs
        assert_eq!(
            nav_status_message("Hunk", 36, &state.doc.hunk_starts, &state.doc.line_map),
            "Hunk 3/6 · b.rs"
        );
        // Cursor on last hunk → Hunk 6/6 · c.rs
        assert_eq!(
            nav_status_message("Hunk", 76, &state.doc.hunk_starts, &state.doc.line_map),
            "Hunk 6/6 · c.rs"
        );
        // Cursor exactly on a hunk_start (header line) → still correct index
        assert_eq!(
            nav_status_message("Hunk", 45, &state.doc.hunk_starts, &state.doc.line_map),
            "Hunk 4/6 · b.rs"
        );
    }

    #[test]
    fn test_file_ix_constructor_boundaries() {
        assert_eq!(FileIx::new(0, 3), Some(FileIx(0)));
        assert_eq!(FileIx::new(2, 3), Some(FileIx(2)));
        assert_eq!(FileIx::new(3, 3), None);
        assert_eq!(FileIx::new(0, 0), None);
    }

    #[test]
    fn test_line_ix_constructor_boundaries() {
        assert_eq!(LineIx::new(0, 10), Some(LineIx(0)));
        assert_eq!(LineIx::new(9, 10), Some(LineIx(9)));
        assert_eq!(LineIx::new(10, 10), None);
        assert_eq!(LineIx::new(0, 0), None);
    }

    #[test]
    fn test_tree_entry_ix_constructor_boundaries() {
        assert_eq!(TreeEntryIx::new(0, 5), Some(TreeEntryIx(0)));
        assert_eq!(TreeEntryIx::new(4, 5), Some(TreeEntryIx(4)));
        assert_eq!(TreeEntryIx::new(5, 5), None);
        assert_eq!(TreeEntryIx::new(0, 0), None);
    }

    #[test]
    fn test_normalize_after_document_swap_clamps_view_scope_when_file_count_shrinks() {
        // Start with 3 files, single-file view on file 2
        let mut state = make_pager_state_for_range(vec![0, 10, 20], 30, Some(2));
        assert_eq!(state.active_file(), Some(2));
        // Simulate document swap: file_starts shrunk to 2 files (remap handles scope normalization)
        let anchor = capture_view_anchor(&state);
        let new_doc = Document {
            lines: vec![String::new(); 20],
            line_map: vec![
                LineInfo {
                    file_idx: 0,
                    path: String::new(),
                    new_lineno: None,
                    old_lineno: None,
                    line_kind: None,
                };
                20
            ],
            file_starts: vec![0, 10],
            hunk_starts: vec![],
        };
        remap_after_document_swap(&mut state, anchor, new_doc, &[]);
        assert_eq!(state.active_file(), None, "view scope should clamp to AllFiles when file 2 no longer exists");
    }

    #[test]
    fn test_tree_focus_invalid_when_tree_empty() {
        let mut state = PagerState::new(
            vec!["x".into(); 5],
            vec![
                LineInfo {
                    file_idx: 0,
                    path: "a".into(),
                    new_lineno: None,
                    old_lineno: None,
                    line_kind: None,
                };
                5
            ],
            vec![0],
            vec![],
            vec![], // empty tree
        );
        assert!(!state.tree_focused());
        assert!(!state.tree_visible);
        state.set_tree_focused(true);
        assert!(!state.tree_focused(), "tree focus must stay false when no entries");
    }

    #[test]
    fn test_compute_connector_prefix_flat() {
        let entries = [
            entry("a.rs", 0, Some(0)),
            entry("b.rs", 0, Some(1)),
            entry("c.rs", 0, Some(2)),
        ];
        let refs: Vec<&TreeEntry> = entries.iter().collect();
        assert_eq!(compute_connector_prefix(&refs, 0), "├── ");
        assert_eq!(compute_connector_prefix(&refs, 1), "├── ");
        assert_eq!(compute_connector_prefix(&refs, 2), "└── ");
    }

    #[test]
    fn test_compute_connector_prefix_nested() {
        // src/
        //   a.rs
        //   b.rs
        // README.md
        let entries = [
            entry("src", 0, None),
            entry("a.rs", 1, Some(0)),
            entry("b.rs", 1, Some(1)),
            entry("README.md", 0, Some(2)),
        ];
        let refs: Vec<&TreeEntry> = entries.iter().collect();
        // src dir: has sibling README.md at depth 0 after it
        assert_eq!(compute_connector_prefix(&refs, 0), "├── ");
        // a.rs: parent (depth 0) continues, sibling b.rs at depth 1 follows
        assert_eq!(compute_connector_prefix(&refs, 1), "│   ├── ");
        // b.rs: parent (depth 0) continues, no more siblings at depth 1
        assert_eq!(compute_connector_prefix(&refs, 2), "│   └── ");
        // README.md: last root entry
        assert_eq!(compute_connector_prefix(&refs, 3), "└── ");
    }

    #[test]
    fn test_build_tree_lines_no_header() {
        let entries = vec![
            entry("a.rs", 0, Some(0)),
            entry("b.rs", 0, Some(1)),
        ];
        let width = compute_tree_width(&entries);
        let (lines, _mapping) = build_tree_lines(&entries, 0, width);
        // First line should be the first tree entry, not a "CHANGED FILES" header
        let first = crate::ansi::strip_ansi(&lines[0]);
        assert!(!first.contains("CHANGED FILES"), "header should be removed");
    }

    #[test]
    fn test_tree_cursor_line_continuous_background() {
        let entries = vec![
            entry("a.rs", 0, Some(0)),
            entry("b.rs", 0, Some(1)),
        ];
        let width = compute_tree_width(&entries);
        let (lines, _) = build_tree_lines(&entries, 0, width);
        let cursor_line = &lines[0];

        // The cursor line must not have RESET followed by a space before the
        // background is re-applied -- that produces an unhighlighted gap between
        // the icon and the filename.
        let forbidden = format!("{} {}", style::RESET, style::BG_TREE_CURSOR);
        assert!(
            !cursor_line.contains(&forbidden),
            "cursor line has a background gap between icon and label:\n{cursor_line}"
        );
    }

    fn make_diff_file(path: &str) -> DiffFile {
        DiffFile {
            old_path: Some(path.to_string()),
            new_path: Some(path.to_string()),
            status: crate::git::diff::FileStatus::Modified,
            hunks: Vec::new(),
        }
    }

    #[test]
    fn test_build_tree_entries_flat_files() {
        let files = vec![make_diff_file("a.rs"), make_diff_file("b.rs")];
        let entries = build_tree_entries(&files);
        assert_eq!(entries.len(), 2);
        assert!(entries.iter().all(|e| e.depth == 0), "all entries should be at depth 0");
        assert!(entries.iter().all(|e| e.file_idx.is_some()), "all entries should be files");
    }

    #[test]
    fn test_build_tree_entries_nested() {
        let files = vec![make_diff_file("src/a.rs"), make_diff_file("src/b.rs")];
        let entries = build_tree_entries(&files);
        // Expect: dir entry "src" then two file entries at depth 1
        let dir_entry = entries.iter().find(|e| e.file_idx.is_none());
        assert!(dir_entry.is_some(), "should have a directory entry");
        assert_eq!(dir_entry.unwrap().label, "src");
        let file_entries: Vec<_> = entries.iter().filter(|e| e.file_idx.is_some()).collect();
        assert_eq!(file_entries.len(), 2);
        assert!(file_entries.iter().all(|e| e.depth == 1), "file entries should be at depth 1");
    }

    #[test]
    fn test_build_tree_entries_single_child_collapse() {
        let files = vec![make_diff_file("src/lib/foo.rs")];
        let entries = build_tree_entries(&files);
        // Single-child directories should collapse: "src" + "lib" -> "src/lib"
        let dir_entry = entries.iter().find(|e| e.file_idx.is_none());
        assert!(dir_entry.is_some(), "should have a collapsed directory entry");
        assert_eq!(dir_entry.unwrap().label, "src/lib", "single-child dirs should collapse");
    }

    #[test]
    fn test_compute_tree_width_empty() {
        assert_eq!(compute_tree_width(&[]), 0);
    }

    #[test]
    fn test_compute_tree_width_capped_at_40() {
        // A very long label should be capped at 40
        let long_label = "a".repeat(60);
        let entries = vec![TreeEntry {
            label: long_label,
            depth: 0,
            file_idx: Some(0),
            status: Some(crate::git::diff::FileStatus::Modified),
            collapsed: false,
        }];
        let width = compute_tree_width(&entries);
        assert_eq!(width, 40, "tree width should be capped at 40");
    }

    #[test]
    fn test_jump_next_finds_first_target_after_top() {
        assert_eq!(jump_next(&[2, 5, 9], 3), Some(5));
    }

    #[test]
    fn test_jump_next_returns_none_when_all_before() {
        assert_eq!(jump_next(&[1, 2], 5), None);
    }

    #[test]
    fn test_jump_prev_finds_last_target_before_top() {
        assert_eq!(jump_prev(&[0, 3, 7], 5), Some(3));
    }

    #[test]
    fn test_jump_prev_returns_none_when_all_after() {
        assert_eq!(jump_prev(&[5, 8], 3), None);
    }

    fn make_pager_state_for_range(
        file_starts: Vec<usize>,
        lines_len: usize,
        active_file: Option<usize>,
    ) -> PagerState {
        let line_map = vec![
            LineInfo {
                file_idx: 0,
                path: String::new(),
                new_lineno: None,
                old_lineno: None,
                line_kind: None,
            };
            lines_len
        ];
        let mut state = PagerState::new(
            vec![String::new(); lines_len],
            line_map,
            file_starts,
            Vec::new(),
            Vec::new(),
        );
        state.set_active_file(active_file);
        state
    }

    #[test]
    fn test_visible_range_no_active_file() {
        let state = make_pager_state_for_range(vec![0, 5], 10, None);
        assert_eq!(visible_range(&state), (0, 10));
    }

    #[test]
    fn test_visible_range_active_file_middle() {
        let state = make_pager_state_for_range(vec![0, 4, 9], 12, Some(1));
        assert_eq!(visible_range(&state), (4, 9));
    }

    #[test]
    fn test_visible_range_active_file_last() {
        let state = make_pager_state_for_range(vec![0, 5], 10, Some(1));
        assert_eq!(visible_range(&state), (5, 10));
    }

    #[test]
    fn test_viewport_bounds_active_file() {
        let state = make_pager_state_for_range(vec![0, 30, 60], 90, Some(1));
        let (range_start, range_end, max_line, max_top) = viewport_bounds(&state, 40);
        assert_eq!((range_start, range_end), (30, 60));
        assert_eq!(max_line, 59);
        assert_eq!(max_top, 30);
    }

    fn make_line_map(kinds: &[Option<LineKind>]) -> Vec<LineInfo> {
        kinds.iter().map(|&kind| LineInfo {
            file_idx: 0,
            path: "test.rs".into(),
            new_lineno: None,
            old_lineno: None,
            line_kind: kind,
        }).collect()
    }

    #[test]
    fn test_is_content_line_true_for_added_deleted_context() {
        let map = make_line_map(&[None, Some(LineKind::Added), Some(LineKind::Deleted), Some(LineKind::Context)]);
        assert!(!is_content_line(&map, 0), "None should not be content");
        assert!(is_content_line(&map, 1), "Added should be content");
        assert!(is_content_line(&map, 2), "Deleted should be content");
        assert!(is_content_line(&map, 3), "Context should be content");
    }

    #[test]
    fn test_next_content_line_skips_header() {
        let map = make_line_map(&[None, None, Some(LineKind::Context)]);
        assert_eq!(next_content_line(&map, 0, 2), 2);
    }

    #[test]
    fn test_next_content_line_returns_from_when_none_found() {
        let map = make_line_map(&[None, None]);
        assert_eq!(next_content_line(&map, 0, 1), 0);
    }

    #[test]
    fn test_prev_content_line_scans_backward() {
        let map = make_line_map(&[Some(LineKind::Context), None, None]);
        assert_eq!(prev_content_line(&map, 2, 0), 0);
    }

    /// Build a line_map with headers at known positions for content-line tests.
    /// Layout (9 entries):
    ///   0: file header  (line_kind = None)
    ///   1: hunk header  (line_kind = None)
    ///   2: context      (line_kind = Some(Context))
    ///   3: added        (line_kind = Some(Added))
    ///   4: deleted      (line_kind = Some(Deleted))
    ///   5: blank sep    (line_kind = None)
    ///   6: hunk header  (line_kind = None)
    ///   7: file header  (line_kind = None)
    ///   8: added        (line_kind = Some(Added))
    fn make_line_map_with_headers() -> Vec<LineInfo> {
        let kinds: Vec<Option<LineKind>> = vec![
            None,                       // 0: file header
            None,                       // 1: hunk header
            Some(LineKind::Context),    // 2: context
            Some(LineKind::Added),      // 3: added
            Some(LineKind::Deleted),    // 4: deleted
            None,                       // 5: blank sep
            None,                       // 6: hunk header
            None,                       // 7: file header
            Some(LineKind::Added),      // 8: added
        ];
        kinds.into_iter().map(|kind| LineInfo {
            file_idx: 0,
            path: "test.rs".into(),
            new_lineno: Some(1),
            old_lineno: None,
            line_kind: kind,
        }).collect()
    }

    #[test]
    fn test_is_content_line() {
        let lm = make_line_map_with_headers();
        assert!(!is_content_line(&lm, 0), "file header is not content");
        assert!(!is_content_line(&lm, 1), "hunk header is not content");
        assert!(is_content_line(&lm, 2), "context is content");
        assert!(is_content_line(&lm, 3), "added is content");
        assert!(is_content_line(&lm, 4), "deleted is content");
        assert!(!is_content_line(&lm, 5), "blank sep is not content");
        assert!(!is_content_line(&lm, 6), "hunk header is not content");
        assert!(!is_content_line(&lm, 7), "file header is not content");
        assert!(is_content_line(&lm, 8), "added is content");
    }

    #[test]
    fn test_j_skips_headers() {
        let lm = make_line_map_with_headers();
        // From content line 4, next content should skip 5/6/7 and land on 8
        let result = next_content_line(&lm, 5, 8);
        assert_eq!(result, 8);
    }

    #[test]
    fn test_k_skips_headers() {
        let lm = make_line_map_with_headers();
        // From content line 8, prev content should skip 7/6/5 and land on 4
        let result = prev_content_line(&lm, 7, 0);
        assert_eq!(result, 4);
    }

    // ---------------------------------------------------------------------------
    // Scrollbar thumb geometry helpers and tests
    // ---------------------------------------------------------------------------

    /// Calls `render_scrollbar_cell` for every row in `0..content_height` and
    /// returns the inclusive `(first_thumb_row, last_thumb_row)` where the
    /// output contains `BG_SCROLLBAR_THUMB`. Panics if no thumb rows are found.
    fn scrollbar_thumb_range(
        content_height: usize,
        range: usize,
        top: usize,
        vis_start: usize,
    ) -> (usize, usize) {
        let vis_end = vis_start + range;
        // Build a line_map large enough to cover vis_end
        let line_map: Vec<LineInfo> = (0..vis_end.max(content_height))
            .map(|_| LineInfo {
                file_idx: 0,
                path: String::new(),
                new_lineno: None,
                old_lineno: None,
                line_kind: None,
            })
            .collect();
        let mut first = None;
        let mut last = None;
        for row in 0..content_height {
            let cell = render_scrollbar_cell(
                row,
                content_height,
                vis_start,
                vis_end,
                top,
                &line_map,
            );
            if cell.contains(style::BG_SCROLLBAR_THUMB) {
                if first.is_none() {
                    first = Some(row);
                }
                last = Some(row);
            }
        }
        (first.expect("no thumb rows found"), last.expect("no thumb rows found"))
    }

    #[test]
    fn test_scrollbar_thumb_fills_screen_when_content_equals_viewport() {
        // content_height=20, vis_start=0, vis_end=20, top=0 -> thumb covers all rows
        let (thumb_start, thumb_end) = scrollbar_thumb_range(20, 20, 0, 0);
        assert_eq!(thumb_start, 0, "thumb should start at row 0");
        assert_eq!(thumb_end, 19, "thumb should end at row 19");
    }

    #[test]
    fn test_scrollbar_thumb_half_height_when_content_double_viewport() {
        // content_height=20, range=40, top=0 -> thumb occupies ~10 rows
        let (thumb_start, thumb_end) = scrollbar_thumb_range(20, 40, 0, 0);
        let height = thumb_end - thumb_start + 1;
        assert!(
            (9..=11).contains(&height),
            "thumb height should be ~10 (within ±1), got {height}"
        );
    }

    #[test]
    fn test_scrollbar_thumb_at_bottom_when_scrolled_to_end() {
        // content_height=20, range=40, top=20 (scrolled fully down) -> thumb in lower half
        let (thumb_start, _thumb_end) = scrollbar_thumb_range(20, 40, 20, 0);
        assert!(
            thumb_start >= 10,
            "thumb should be in the lower half when scrolled to end, got thumb_start={thumb_start}"
        );
    }

    #[test]
    fn test_scrollbar_thumb_minimum_height_one_row() {
        // With an enormous range the thumb is tiny, but the .max(thumb_start+1) guard
        // ensures at least 1 row is covered.
        let (thumb_start, thumb_end) = scrollbar_thumb_range(20, 10000, 0, 0);
        assert!(
            thumb_end >= thumb_start,
            "thumb must occupy at least 1 row (min-height guard)"
        );
    }

    #[test]
    fn test_scrollbar_no_crash_on_zero_range() {
        // vis_start == vis_end -> range = 0; must return a track cell without panicking
        let line_map: Vec<LineInfo> = (0..5)
            .map(|_| LineInfo {
                file_idx: 0,
                path: String::new(),
                new_lineno: None,
                old_lineno: None,
                line_kind: None,
            })
            .collect();
        let cell = render_scrollbar_cell(0, 20, 5, 5, 0, &line_map);
        assert!(
            cell.contains(style::BG_SCROLLBAR_TRACK),
            "zero-range should return a track cell"
        );
    }

    #[test]
    fn test_scrollbar_no_crash_on_empty_line_map() {
        // Empty line_map; must return a valid cell without panicking
        let cell = render_scrollbar_cell(0, 20, 0, 40, 0, &[]);
        assert!(
            cell.contains(style::BG_SCROLLBAR_TRACK) || cell.contains(style::BG_SCROLLBAR_THUMB),
            "empty line_map should return a valid scrollbar cell without panicking"
        );
    }

    // ---------------------------------------------------------------------------
    // Tree status symbol tests (chunk-05)
    // ---------------------------------------------------------------------------

    fn entry_with_status(label: &str, depth: usize, file_idx: usize, status: FileStatus) -> TreeEntry {
        TreeEntry {
            label: label.to_string(),
            depth,
            file_idx: Some(file_idx),
            status: Some(status),
            collapsed: false,
        }
    }

    #[test]
    fn test_tree_status_symbol_modified() {
        let entries = vec![entry_with_status("foo.rs", 0, 0, FileStatus::Modified)];
        let width = compute_tree_width(&entries);
        let (lines, _) = build_tree_lines(&entries, 0, width);
        let stripped = crate::ansi::strip_ansi(&lines[0]);
        assert!(stripped.contains("M "), "modified entry should contain 'M ': {stripped:?}");
    }

    #[test]
    fn test_tree_status_symbol_added() {
        let entries = vec![entry_with_status("foo.rs", 0, 0, FileStatus::Added)];
        let width = compute_tree_width(&entries);
        let (lines, _) = build_tree_lines(&entries, 0, width);
        let stripped = crate::ansi::strip_ansi(&lines[0]);
        assert!(stripped.contains("A "), "added entry should contain 'A ': {stripped:?}");
    }

    #[test]
    fn test_tree_status_symbol_deleted() {
        let entries = vec![entry_with_status("foo.rs", 0, 0, FileStatus::Deleted)];
        let width = compute_tree_width(&entries);
        let (lines, _) = build_tree_lines(&entries, 0, width);
        let stripped = crate::ansi::strip_ansi(&lines[0]);
        assert!(stripped.contains("D "), "deleted entry should contain 'D ': {stripped:?}");
    }

    #[test]
    fn test_tree_status_symbol_renamed() {
        let entries = vec![entry_with_status("foo.rs", 0, 0, FileStatus::Renamed)];
        let width = compute_tree_width(&entries);
        let (lines, _) = build_tree_lines(&entries, 0, width);
        let stripped = crate::ansi::strip_ansi(&lines[0]);
        assert!(stripped.contains("R "), "renamed entry should contain 'R ': {stripped:?}");
    }

    #[test]
    fn test_tree_status_symbol_untracked() {
        let entries = vec![entry_with_status("foo.rs", 0, 0, FileStatus::Untracked)];
        let width = compute_tree_width(&entries);
        let (lines, _) = build_tree_lines(&entries, 0, width);
        let stripped = crate::ansi::strip_ansi(&lines[0]);
        assert!(stripped.contains("? "), "untracked entry should contain '? ': {stripped:?}");
    }

    #[test]
    fn test_tree_status_symbol_directory() {
        let entries = vec![
            TreeEntry {
                label: "src".to_string(),
                depth: 0,
                file_idx: None,
                status: None,
                collapsed: false,
            },
        ];
        let width = compute_tree_width(&entries);
        let (lines, _) = build_tree_lines(&entries, 0, width);
        let stripped = crate::ansi::strip_ansi(&lines[0]);
        assert!(!stripped.contains("M "), "directory must not show M: {stripped:?}");
        assert!(!stripped.contains("A "), "directory must not show A: {stripped:?}");
        assert!(!stripped.contains("D "), "directory must not show D: {stripped:?}");
        assert!(!stripped.contains("R "), "directory must not show R: {stripped:?}");
        assert!(!stripped.contains("? "), "directory must not show ?: {stripped:?}");
    }

    #[test]
    fn test_compute_tree_width_includes_status_chars() {
        let entry_with = entry_with_status("foo.rs", 0, 0, FileStatus::Modified);
        let entry_without = TreeEntry {
            label: "foo.rs".to_string(),
            depth: 0,
            file_idx: Some(0),
            status: None,
            collapsed: false,
        };
        let width_with = compute_tree_width(&[entry_with]);
        let width_without = compute_tree_width(&[entry_without]);
        assert_eq!(
            width_with,
            width_without + 2,
            "status symbol adds 2 columns (char + space): with={width_with}, without={width_without}"
        );
    }

    // ---------------------------------------------------------------------------
    // Insta snapshot tests
    // ---------------------------------------------------------------------------

    #[test]
    fn snapshot_tree_entries_flat() {
        let files = vec![make_diff_file("a.rs"), make_diff_file("b.rs"), make_diff_file("c.rs")];
        assert_debug_snapshot!(build_tree_entries(&files));
    }

    #[test]
    fn snapshot_tree_entries_nested() {
        let files = vec![
            make_diff_file("src/lib.rs"),
            make_diff_file("src/main.rs"),
            make_diff_file("README.md"),
        ];
        assert_debug_snapshot!(build_tree_entries(&files));
    }

    #[test]
    fn snapshot_tree_entries_single_child_collapse() {
        let files = vec![
            make_diff_file("src/lib/foo.rs"),
            make_diff_file("src/lib/bar.rs"),
            make_diff_file("tests/integration.rs"),
        ];
        assert_debug_snapshot!(build_tree_entries(&files));
    }

    #[test]
    fn snapshot_tree_entries_with_status() {
        let files = vec![
            DiffFile { old_path: Some("a.rs".into()), new_path: Some("a.rs".into()), status: FileStatus::Modified, hunks: vec![] },
            DiffFile { old_path: None, new_path: Some("b.rs".into()), status: FileStatus::Added, hunks: vec![] },
            DiffFile { old_path: Some("c.rs".into()), new_path: None, status: FileStatus::Deleted, hunks: vec![] },
            DiffFile { old_path: Some("d.rs".into()), new_path: Some("e.rs".into()), status: FileStatus::Renamed, hunks: vec![] },
            DiffFile { old_path: None, new_path: Some("f.rs".into()), status: FileStatus::Untracked, hunks: vec![] },
        ];
        assert_debug_snapshot!(build_tree_entries(&files));
    }

    fn strip(s: &str) -> String {
        crate::ansi::strip_ansi(s)
    }

    #[test]
    fn snapshot_tree_lines_flat() {
        let entries = vec![
            entry("a.rs", 0, Some(0)),
            entry("b.rs", 0, Some(1)),
            entry("c.rs", 0, Some(2)),
        ];
        let width = compute_tree_width(&entries);
        let (lines, _) = build_tree_lines(&entries, 0, width);
        let stripped: Vec<String> = lines.iter().map(|l| strip(l)).collect();
        assert_snapshot!(stripped.join("\n"));
    }

    #[test]
    fn snapshot_tree_lines_nested() {
        let files = vec![
            make_diff_file("src/lib.rs"),
            make_diff_file("src/main.rs"),
            make_diff_file("README.md"),
        ];
        let entries = build_tree_entries(&files);
        let width = compute_tree_width(&entries);
        let (lines, _) = build_tree_lines(&entries, 0, width);
        let stripped: Vec<String> = lines.iter().map(|l| strip(l)).collect();
        assert_snapshot!(stripped.join("\n"));
    }

    // ---------------------------------------------------------------------------
    // Keybinding snapshot tests (chunk-02)
    // ---------------------------------------------------------------------------

    #[test]
    fn key_j_next_content_line() {
        let mut state = make_keybinding_state();
        state.cursor_line = 1;
        handle_key(&mut state, Key::Char('j'), 40, 40, &[]);
        assert_debug_snapshot!(StateSnapshot::from(&state));
    }

    #[test]
    fn key_k_prev_content_line() {
        let mut state = make_keybinding_state();
        state.cursor_line = 6;
        handle_key(&mut state, Key::Char('k'), 40, 40, &[]);
        assert_debug_snapshot!(StateSnapshot::from(&state));
    }

    #[test]
    fn key_j_skips_headers() {
        let mut state = make_keybinding_state();
        state.cursor_line = 4; // next is 5 which is a header, should skip to 6
        handle_key(&mut state, Key::Char('j'), 40, 40, &[]);
        assert_debug_snapshot!(StateSnapshot::from(&state));
    }

    #[test]
    fn key_g_jumps_to_first_content() {
        let mut state = make_keybinding_state();
        state.cursor_line = 15;
        handle_key(&mut state, Key::Char('g'), 40, 40, &[]);
        assert_debug_snapshot!(StateSnapshot::from(&state));
    }

    #[test]
    #[allow(non_snake_case)]
    fn key_G_jumps_to_last_content() {
        let mut state = make_keybinding_state();
        state.cursor_line = 1;
        handle_key(&mut state, Key::Char('G'), 40, 40, &[]);
        assert_debug_snapshot!(StateSnapshot::from(&state));
    }

    #[test]
    fn key_ctrl_d_half_page_down() {
        let mut state = make_keybinding_state();
        state.cursor_line = 1;
        handle_key(&mut state, Key::CtrlD, 40, 40, &[]);
        assert_debug_snapshot!(StateSnapshot::from(&state));
    }

    #[test]
    fn key_ctrl_u_half_page_up() {
        let mut state = make_keybinding_state();
        state.cursor_line = 25;
        handle_key(&mut state, Key::CtrlU, 40, 40, &[]);
        assert_debug_snapshot!(StateSnapshot::from(&state));
    }

    #[test]
    fn key_d_next_hunk_same_file() {
        let mut state = make_keybinding_state();
        state.cursor_line = 8;
        handle_key(&mut state, Key::Char('d'), 40, 40, &[]);
        assert_debug_snapshot!(StateSnapshot::from(&state));
    }

    #[test]
    fn key_u_prev_hunk_same_file() {
        let mut state = make_keybinding_state();
        state.cursor_line = 16;
        handle_key(&mut state, Key::Char('u'), 40, 40, &[]);
        assert_debug_snapshot!(StateSnapshot::from(&state));
    }

    #[test]
    fn key_u_prev_hunk_from_first_content_line() {
        let mut state = make_keybinding_state();
        state.cursor_line = 16; // first content line after hunk header at 15
        state.set_active_file(Some(0));
        handle_key(&mut state, Key::Char('u'), 40, 40, &[]);
        assert_eq!(state.cursor_line, 6);
    }

    #[test]
    fn key_u_cross_file_from_first_hunk() {
        let mut state = make_keybinding_state();
        state.set_active_file(Some(1));
        state.cursor_line = 36; // first content line of file 1's first hunk (header at 35)
        handle_key(&mut state, Key::Char('u'), 40, 40, &[]);
        assert_eq!(state.cursor_line, 16);
    }

    #[test]
    fn key_u_tree_focused_from_first_content_line() {
        let mut state = make_keybinding_state();
        state.set_tree_focused(true);
        state.cursor_line = 16;
        state.set_active_file(Some(0));
        handle_key(&mut state, Key::Char('u'), 40, 40, &[]);
        assert_eq!(state.cursor_line, 6);
    }

    #[test]
    fn key_d_cross_file_boundary() {
        let mut state = make_keybinding_state();
        state.cursor_line = 16;
        handle_key(&mut state, Key::Char('d'), 40, 40, &[]);
        assert_debug_snapshot!(StateSnapshot::from(&state));
    }

    #[test]
    fn key_u_cross_file_boundary() {
        let mut state = make_keybinding_state();
        state.set_active_file(Some(1));
        state.cursor_line = 36;
        handle_key(&mut state, Key::Char('u'), 40, 40, &[]);
        assert_eq!(state.cursor_line, 16);
    }

    #[test]
    fn key_d_next_hunk_scrolloff_binding() {
        let mut state = make_keybinding_state();
        state.cursor_line = 6;
        handle_key(&mut state, Key::Char('d'), 15, 40, &[]);
        assert_debug_snapshot!(StateSnapshot::from(&state));
    }

    #[test]
    #[allow(non_snake_case)]
    fn key_D_next_file() {
        let mut state = make_keybinding_state();
        state.set_active_file(Some(0));
        handle_key(&mut state, Key::Char('D'), 40, 40, &[]);
        assert_debug_snapshot!(StateSnapshot::from(&state));
    }

    #[test]
    #[allow(non_snake_case)]
    fn key_U_prev_file() {
        let mut state = make_keybinding_state();
        state.set_active_file(Some(1));
        state.cursor_line = 31;
        handle_key(&mut state, Key::Char('U'), 40, 40, &[]);
        assert_debug_snapshot!(StateSnapshot::from(&state));
    }

    #[test]
    #[allow(non_snake_case)]
    fn key_U_no_active_file_stuck_cursor() {
        let mut state = make_keybinding_state();
        state.set_active_file(None);
        state.cursor_line = 31; // first content line after file_start=30
        handle_key(&mut state, Key::Char('U'), 50, 40, &[]);
        assert_debug_snapshot!(StateSnapshot::from(&state));
    }

    #[test]
    fn key_d_no_active_file_does_not_stick() {
        let mut state = make_keybinding_state();
        state.set_active_file(None);
        state.cursor_line = 5; // on a hunk header
        handle_key(&mut state, Key::Char('d'), 40, 40, &[]);
        assert_debug_snapshot!(StateSnapshot::from(&state));
    }

    #[test]
    fn key_next_file_no_active_file_does_not_stick() {
        let mut state = make_keybinding_state();
        state.set_active_file(None);
        state.cursor_line = 0; // on a file header
        handle_key(&mut state, Key::Char('D'), 40, 40, &[]);
        assert_debug_snapshot!(StateSnapshot::from(&state));
    }

    #[test]
    fn key_d_at_last_hunk_is_noop() {
        let mut state = make_keybinding_state();
        state.set_active_file(None);
        state.cursor_line = 76; // after last hunk_start=75
        handle_key(&mut state, Key::Char('d'), 40, 40, &[]);
        assert_debug_snapshot!(StateSnapshot::from(&state));
    }

    #[test]
    fn key_u_at_first_hunk_is_noop() {
        let mut state = make_keybinding_state();
        state.set_active_file(None);
        state.cursor_line = 6; // after first hunk_start=5
        handle_key(&mut state, Key::Char('u'), 40, 40, &[]);
        assert_debug_snapshot!(StateSnapshot::from(&state));
    }

    #[test]
    #[allow(non_snake_case)]
    fn key_D_at_last_file_is_noop() {
        let mut state = make_keybinding_state();
        state.set_active_file(None);
        state.cursor_line = 66; // in last file
        handle_key(&mut state, Key::Char('D'), 40, 40, &[]);
        assert_debug_snapshot!(StateSnapshot::from(&state));
    }

    #[test]
    #[allow(non_snake_case)]
    fn key_U_at_first_file_is_noop() {
        let mut state = make_keybinding_state();
        state.set_active_file(None);
        state.cursor_line = 1; // in first file
        handle_key(&mut state, Key::Char('U'), 50, 40, &[]);
        assert_debug_snapshot!(StateSnapshot::from(&state));
    }

    // ---------------------------------------------------------------------------
    // Single-file mode still constrains file-level navigation (D/U),
    // while hunk-level navigation (d/u) is global across files.
    // ---------------------------------------------------------------------------

    #[test]
    fn key_d_single_file_jumps_to_next_file_hunk() {
        let mut state = make_keybinding_state();
        state.set_active_file(Some(0)); // a.rs: lines 0-29, hunks at 5, 15
        state.cursor_line = 16; // past last hunk of file 0
        handle_key(&mut state, Key::Char('d'), 40, 40, &[]);
        // d/u ignore single-file scope and navigate hunks globally.
        assert_eq!(state.cursor_line, 36);
        assert_eq!(state.active_file(), Some(1));
    }

    #[test]
    fn key_u_single_file_jumps_to_prev_file_hunk() {
        let mut state = make_keybinding_state();
        state.set_active_file(Some(1)); // b.rs: lines 30-59, hunks at 35, 45
        state.cursor_line = 36; // first content of file 1's first hunk
        handle_key(&mut state, Key::Char('u'), 40, 40, &[]);
        assert_eq!(state.cursor_line, 16);
        assert_eq!(state.active_file(), Some(0));
    }

    #[test]
    fn key_next_file_single_file_is_noop() {
        let mut state = make_keybinding_state();
        state.set_active_file(Some(0)); // only file 0 visible
        state.cursor_line = 1;
        handle_key(&mut state, Key::Char('D'), 40, 40, &[]);
        // D should not jump to file 1 — no other file_starts in range
        assert_debug_snapshot!(StateSnapshot::from(&state));
    }

    #[test]
    fn key_prev_file_single_file_is_noop() {
        let mut state = make_keybinding_state();
        state.set_active_file(Some(2)); // only file 2 visible
        state.cursor_line = 61;
        handle_key(&mut state, Key::Char('U'), 40, 40, &[]);
        // U should not jump to file 1 — no other file_starts in range
        assert_debug_snapshot!(StateSnapshot::from(&state));
    }

    #[test]
    fn key_d_single_file_within_file_works() {
        let mut state = make_keybinding_state();
        state.set_active_file(Some(0)); // a.rs: hunks at 5, 15
        state.cursor_line = 6; // just past first hunk
        handle_key(&mut state, Key::Char('d'), 40, 40, &[]);
        // Should jump to hunk at 15 (within same file)
        assert_eq!(state.cursor_line, 16);
    }

    #[test]
    fn key_d_tree_focused_single_file_jumps_globally() {
        let mut state = make_keybinding_state();
        state.set_tree_focused(true);
        state.set_active_file(Some(0));
        state.cursor_line = 16; // past last hunk of file 0
        handle_key(&mut state, Key::Char('d'), 40, 40, &[]);
        assert_eq!(state.cursor_line, 36);
        assert_eq!(state.active_file(), Some(1));
        assert_eq!(state.tree_cursor(), file_idx_to_entry_idx(&state.tree_entries, 1));
    }

    #[test]
    fn key_u_tree_focused_single_file_jumps_globally() {
        let mut state = make_keybinding_state();
        state.set_tree_focused(true);
        state.set_active_file(Some(1)); // b.rs: hunks at 35, 45
        state.cursor_line = 36;
        handle_key(&mut state, Key::Char('u'), 40, 40, &[]);
        assert_eq!(state.cursor_line, 16);
        assert_eq!(state.active_file(), Some(0));
        assert_eq!(state.tree_cursor(), file_idx_to_entry_idx(&state.tree_entries, 0));
    }

    #[test]
    fn key_d_single_file_clamps_top_line_to_active_file_range() {
        let mut state = make_keybinding_state();
        state.set_active_file(Some(0));
        state.cursor_line = 16; // next global hunk lands in file 1
        handle_key(&mut state, Key::Char('d'), 40, 40, &[]);
        let (range_start, range_end) = visible_range(&state);
        let max_top = range_end.saturating_sub(40).max(range_start);
        assert!(state.top_line >= range_start);
        assert!(state.top_line <= max_top);
    }

    #[test]
    fn key_u_tree_focused_single_file_clamps_top_line_to_active_file_range() {
        let mut state = make_keybinding_state();
        state.set_tree_focused(true);
        state.set_active_file(Some(1));
        state.cursor_line = 36; // previous global hunk lands in file 0
        handle_key(&mut state, Key::Char('u'), 40, 40, &[]);
        let (range_start, range_end) = visible_range(&state);
        let max_top = range_end.saturating_sub(40).max(range_start);
        assert!(state.top_line >= range_start);
        assert!(state.top_line <= max_top);
    }

    #[test]
    fn key_slash_enters_search() {
        let mut state = make_keybinding_state();
        handle_key(&mut state, Key::Char('/'), 40, 40, &[]);
        assert_debug_snapshot!(StateSnapshot::from(&state));
    }

    #[test]
    fn key_question_enters_help() {
        let mut state = make_keybinding_state();
        handle_key(&mut state, Key::Char('?'), 40, 40, &[]);
        assert_debug_snapshot!(StateSnapshot::from(&state));
    }

    #[test]
    fn key_v_enters_visual() {
        let mut state = make_keybinding_state();
        handle_key(&mut state, Key::Char('v'), 40, 40, &[]);
        assert_debug_snapshot!(StateSnapshot::from(&state));
    }

    #[test]
    fn key_esc_exits_visual() {
        let mut state = make_keybinding_state();
        state.mode = Mode::Visual;
        state.visual_anchor = 10;
        handle_key(&mut state, Key::Escape, 40, 40, &[]);
        assert_debug_snapshot!(StateSnapshot::from(&state));
    }

    // ---------------------------------------------------------------------------
    // Tree-focused defocus and navigation snapshot tests (chunk-03)
    // ---------------------------------------------------------------------------

    #[test]
    fn key_h_in_tree_defocuses() {
        let mut state = make_keybinding_state();
        state.set_tree_focused(true);
        handle_key(&mut state, Key::Char('h'), 40, 40, &[]);
        assert_debug_snapshot!(StateSnapshot::from(&state));
    }

    #[test]
    fn key_esc_in_tree_defocuses() {
        let mut state = make_keybinding_state();
        state.set_tree_focused(true);
        handle_key(&mut state, Key::Escape, 40, 40, &[]);
        assert_debug_snapshot!(StateSnapshot::from(&state));
    }

    #[test]
    fn key_tab_in_tree_defocuses() {
        let mut state = make_keybinding_state();
        state.set_tree_focused(true);
        handle_key(&mut state, Key::Tab, 40, 40, &[]);
        assert_debug_snapshot!(StateSnapshot::from(&state));
    }

    #[test]
    fn key_1_in_tree_defocuses() {
        let mut state = make_keybinding_state();
        state.set_tree_focused(true);
        handle_key(&mut state, Key::Char('1'), 40, 40, &[]);
        assert_debug_snapshot!(StateSnapshot::from(&state));
    }

    #[test]
    fn key_j_in_tree_next_entry() {
        let mut state = make_keybinding_state();
        state.set_tree_focused(true);
        state.set_tree_cursor(0);
        let initial_top = state.top_line;
        let initial_cursor = state.cursor_line;
        handle_key(&mut state, Key::Char('j'), 40, 40, &[]);
        assert_eq!(state.tree_cursor(), 1);
        assert_eq!(state.top_line, initial_top);
        assert_eq!(state.cursor_line, initial_cursor);
    }

    #[test]
    fn key_k_in_tree_prev_entry() {
        let mut state = make_keybinding_state();
        state.set_tree_focused(true);
        // Move to the second entry first
        state.set_tree_cursor(state.tree_visible_to_entry[1]);
        let initial_top = state.top_line;
        let initial_cursor = state.cursor_line;
        handle_key(&mut state, Key::Char('k'), 40, 40, &[]);
        assert_eq!(state.tree_cursor(), 0);
        assert_eq!(state.top_line, initial_top);
        assert_eq!(state.cursor_line, initial_cursor);
    }

    #[test]
    fn key_g_in_tree_first_entry() {
        let mut state = make_keybinding_state();
        state.set_tree_focused(true);
        // Start at the last entry
        state.set_tree_cursor(*state.tree_visible_to_entry.last().unwrap());
        handle_key(&mut state, Key::Char('g'), 40, 40, &[]);
        assert_debug_snapshot!(StateSnapshot::from(&state));
    }

    #[test]
    #[allow(non_snake_case)]
    fn key_G_in_tree_last_entry() {
        let mut state = make_keybinding_state();
        state.set_tree_focused(true);
        // Start at the first entry
        state.set_tree_cursor(state.tree_visible_to_entry[0]);
        handle_key(&mut state, Key::Char('G'), 40, 40, &[]);
        assert_debug_snapshot!(StateSnapshot::from(&state));
    }

    #[test]
    fn key_enter_on_file_in_tree() {
        let mut state = make_keybinding_state();
        state.set_tree_focused(true);
        // Cursor on the second file entry (b.rs, file_idx=1)
        state.set_tree_cursor(state.tree_visible_to_entry[1]);
        handle_key(&mut state, Key::Enter, 40, 40, &[]);
        assert_debug_snapshot!(StateSnapshot::from(&state));
    }

    #[test]
    fn key_enter_on_dir_in_tree() {
        // Build a state with nested paths so tree_entries[0] is a directory
        let mut state = make_keybinding_state();
        state.tree_entries = vec![
            TreeEntry {
                label: "src".to_string(),
                depth: 0,
                file_idx: None,
                status: None,
                collapsed: false,
            },
            entry("a.rs", 1, Some(0)),
            entry("b.rs", 1, Some(1)),
        ];
        state.tree_width = compute_tree_width(&state.tree_entries);
        let (tl, tv) = build_tree_lines(&state.tree_entries, 0, state.tree_width);
        state.tree_lines = tl;
        state.tree_visible_to_entry = tv;
        state.set_tree_focused(true);
        state.set_tree_cursor(0); // directory entry
        handle_key(&mut state, Key::Enter, 40, 40, &[]);
        // Verify directory collapsed toggled
        assert!(state.tree_entries[0].collapsed);
        assert_debug_snapshot!(StateSnapshot::from(&state));
    }

    // ---------------------------------------------------------------------------
    // Tree management and toggle snapshot tests -- normal mode (chunk-03)
    // ---------------------------------------------------------------------------

    #[test]
    fn key_e_opens_and_focuses_tree() {
        let mut state = make_keybinding_state();
        state.tree_visible = false;
        state.set_tree_focused(false);
        let files = vec![make_diff_file("a.rs"), make_diff_file("b.rs"), make_diff_file("c.rs")];
        handle_key(&mut state, Key::Char('e'), 40, 40, &files);
        assert_debug_snapshot!(StateSnapshot::from(&state));
    }

    #[test]
    fn key_e_focuses_open_tree() {
        let mut state = make_keybinding_state();
        state.tree_visible = true;
        state.set_tree_focused(false);
        handle_key(&mut state, Key::Char('e'), 40, 40, &[]);
        assert_debug_snapshot!(StateSnapshot::from(&state));
    }

    #[test]
    fn key_e_closes_focused_tree() {
        let mut state = make_keybinding_state();
        state.tree_visible = true;
        state.set_tree_focused(true);
        handle_key(&mut state, Key::Char('e'), 40, 40, &[]);
        assert_debug_snapshot!(StateSnapshot::from(&state));
    }

    #[test]
    fn e_close_tree_preserves_active_file() {
        let mut state = make_keybinding_state();
        state.set_active_file(Some(1));
        state.tree_visible = true;
        state.set_tree_focused(true);
        handle_key(&mut state, Key::Char('e'), 40, 40, &[]);
        assert_eq!(state.active_file(), Some(1), "single-file view must be preserved");
        assert!(!state.tree_visible);
        assert!(!state.tree_focused());
    }

    #[test]
    fn e_close_tree_preserves_active_file_none() {
        let mut state = make_keybinding_state();
        state.set_active_file(None);
        state.tree_visible = true;
        state.set_tree_focused(true);
        handle_key(&mut state, Key::Char('e'), 40, 40, &[]);
        assert_eq!(state.active_file(), None, "active_file must stay None");
    }

    #[test]
    fn key_tab_focuses_tree() {
        let mut state = make_keybinding_state();
        state.tree_visible = true;
        state.set_tree_focused(false);
        handle_key(&mut state, Key::Tab, 40, 40, &[]);
        assert_debug_snapshot!(StateSnapshot::from(&state));
    }

    #[test]
    fn key_l_shows_and_focuses_tree() {
        let mut state = make_keybinding_state();
        state.tree_visible = false;
        state.set_tree_focused(false);
        let files = vec![make_diff_file("a.rs"), make_diff_file("b.rs"), make_diff_file("c.rs")];
        handle_key(&mut state, Key::Char('l'), 40, 40, &files);
        assert_debug_snapshot!(StateSnapshot::from(&state));
    }

    #[test]
    fn key_a_toggles_off_single_file() {
        let mut state = make_keybinding_state();
        state.set_active_file(Some(0));
        handle_key(&mut state, Key::Char('a'), 40, 40, &[]);
        assert_debug_snapshot!(StateSnapshot::from(&state));
    }

    #[test]
    fn key_a_toggles_on_single_file() {
        let mut state = make_keybinding_state();
        state.set_active_file(None);
        handle_key(&mut state, Key::Char('a'), 40, 40, &[]);
        assert_debug_snapshot!(StateSnapshot::from(&state));
    }

    #[test]
    fn key_space_is_noop_for_full_context_toggle() {
        let mut state = make_keybinding_state();
        state.full_context = false;
        handle_key(&mut state, Key::Char(' '), 40, 40, &[]);
        assert!(!state.full_context);
    }

    #[test]
    fn key_space_is_noop_for_context_toggle() {
        let mut state = make_keybinding_state();
        state.full_context = true;
        handle_key(&mut state, Key::Char(' '), 40, 40, &[]);
        assert!(state.full_context);
    }

    #[test]
    fn key_z_toggles_full_context() {
        let mut state = make_keybinding_state();
        handle_key(&mut state, Key::Char('z'), 40, 40, &[]);
        assert!(state.full_context);
    }

    #[test]
    fn key_z_toggles_hunk_context() {
        let mut state = make_keybinding_state();
        state.full_context = true;
        handle_key(&mut state, Key::Char('z'), 40, 40, &[]);
        assert!(!state.full_context);
    }

    // Regression: keymap/help/docs consistency (chunk-04)
    #[test]
    fn keymap_full_context_toggle_is_z() {
        use super::KeyContext;
        assert_eq!(
            keymap_lookup(Key::Char('z'), KeyContext::Normal),
            Some(ActionId::ToggleFullContext)
        );
        assert_eq!(keymap_lookup(Key::Char(' '), KeyContext::Normal), None);
    }

    #[test]
    fn help_includes_full_context_toggle_z() {
        let help = keymap_help_lines();
        let has_z = help.iter().any(|l| l.contains('z'));
        let has_label = help.iter().any(|l| l.contains("Toggle full file context"));
        assert!(has_z, "help must show z for full-context toggle");
        assert!(has_label, "help must describe full file context toggle");
    }

    #[test]
    fn help_includes_all_primary_runtime_actions() {
        let help_text = keymap_help_lines().join(" ");
        let required = [
            "d",
            "Next hunk",
            "u",
            "Previous hunk",
            "D",
            "Next file",
            "U",
            "Previous file",
            "a",
            "Toggle single file",
            "z",
            "Toggle full file context",
            "/",
            "Search",
            "n",
            "Next match",
            "N",
            "Previous match",
            "e",
            "Toggle tree panel",
            "v",
            "Enter visual mode",
            "E",
            "Open in editor",
            "q",
            "Quit",
        ];
        for s in required {
            assert!(
                help_text.contains(s),
                "help must include runtime action: {s:?}"
            );
        }
    }

    #[test]
    fn test_initial_state_no_active_file() {
        let state = make_keybinding_state();
        assert_debug_snapshot!(StateSnapshot::from(&state));
    }

    #[test]
    fn test_tree_j_without_active_file_moves_tree_only() {
        let mut state = make_keybinding_state();
        state.set_tree_focused(true);
        let initial_top = state.top_line;
        let initial_cursor = state.cursor_line;
        handle_key(&mut state, Key::Char('j'), 40, 40, &[]);
        assert_eq!(state.tree_cursor(), 1);
        assert_eq!(state.top_line, initial_top);
        assert_eq!(state.cursor_line, initial_cursor);
    }

    #[test]
    fn test_tree_j_single_file_moves_tree_only() {
        let mut state = make_keybinding_state();
        state.set_tree_focused(true);
        state.set_active_file(Some(0));
        let initial_top = state.top_line;
        let initial_cursor = state.cursor_line;
        handle_key(&mut state, Key::Char('j'), 40, 40, &[]);
        assert_eq!(state.tree_cursor(), 1);
        assert_eq!(state.top_line, initial_top);
        assert_eq!(state.cursor_line, initial_cursor);
        assert_eq!(state.active_file(), Some(0));
    }

    #[test]
    fn test_tree_k_without_active_file_moves_tree_only() {
        let mut state = make_keybinding_state();
        state.set_tree_focused(true);
        // Start at second file entry (tree_cursor = 1, which is b.rs / file 1)
        state.set_tree_cursor(1);
        let (tl, tv) = build_tree_lines(&state.tree_entries, state.tree_cursor(), state.tree_width);
        state.tree_lines = tl;
        state.tree_visible_to_entry = tv;
        let initial_top = state.top_line;
        let initial_cursor = state.cursor_line;
        handle_key(&mut state, Key::Char('k'), 40, 40, &[]);
        assert_eq!(state.tree_cursor(), 0);
        assert_eq!(state.top_line, initial_top);
        assert_eq!(state.cursor_line, initial_cursor);
    }

    #[test]
    fn test_tree_k_single_file_moves_tree_only() {
        let mut state = make_keybinding_state();
        state.set_tree_focused(true);
        state.set_active_file(Some(1));
        state.set_tree_cursor(1);
        state.top_line = 30;
        state.cursor_line = 31;
        let (tl, tv) = build_tree_lines(&state.tree_entries, state.tree_cursor(), state.tree_width);
        state.tree_lines = tl;
        state.tree_visible_to_entry = tv;
        let initial_top = state.top_line;
        let initial_cursor = state.cursor_line;
        handle_key(&mut state, Key::Char('k'), 40, 40, &[]);
        let (vis_start, vis_end) = visible_range(&state);
        assert_eq!(state.tree_cursor(), 0);
        assert_eq!(state.top_line, initial_top);
        assert_eq!(state.cursor_line, initial_cursor);
        assert!(state.cursor_line >= vis_start && state.cursor_line < vis_end);
        assert!(is_content_line(&state.doc.line_map, state.cursor_line));
        assert_eq!(state.active_file(), Some(1));
    }

    #[test]
    fn test_tree_enter_scrolls_without_active_file() {
        let mut state = make_keybinding_state();
        state.set_tree_focused(true);
        // Move cursor to second file entry (b.rs / file 1)
        state.set_tree_cursor(1);
        let (tl, tv) = build_tree_lines(&state.tree_entries, state.tree_cursor(), state.tree_width);
        state.tree_lines = tl;
        state.tree_visible_to_entry = tv;
        handle_key(&mut state, Key::Enter, 40, 40, &[]);
        assert_debug_snapshot!(StateSnapshot::from(&state));
    }

    #[test]
    fn test_tree_enter_single_file_switches_active_file() {
        let mut state = make_keybinding_state();
        state.set_tree_focused(true);
        state.set_active_file(Some(0));
        state.set_tree_cursor(1);
        let (tl, tv) = build_tree_lines(&state.tree_entries, state.tree_cursor(), state.tree_width);
        state.tree_lines = tl;
        state.tree_visible_to_entry = tv;
        handle_key(&mut state, Key::Enter, 40, 40, &[]);
        assert_eq!(state.active_file(), Some(1));
        assert_eq!(state.top_line, 30);
        assert!(state.cursor_line >= 30 && state.cursor_line < 60);
    }

    #[test]
    fn test_a_still_toggles_single_file() {
        let mut state = make_keybinding_state();
        // Default state has active_file: None; pressing 'a' should set it
        handle_key(&mut state, Key::Char('a'), 40, 40, &[]);
        assert_eq!(state.active_file(), Some(0));
        // Press 'a' again to toggle off
        handle_key(&mut state, Key::Char('a'), 40, 40, &[]);
        assert_eq!(state.active_file(), None);
        assert_debug_snapshot!(StateSnapshot::from(&state));
    }

    // ---------------------------------------------------------------------------
    // skip_headers pass-through tests (chunk-03)
    // ---------------------------------------------------------------------------

    fn make_two_file_diff() -> Vec<DiffFile> {
        let raw = "\
diff --git a/a.txt b/a.txt
--- a/a.txt
+++ b/a.txt
@@ -1,1 +1,2 @@
 first
+second
diff --git a/b.txt b/b.txt
--- a/b.txt
+++ b/b.txt
@@ -1,2 +1,1 @@
 keep
-remove
";
        crate::git::diff::parse(raw)
    }

    fn make_pager_state_from_files(files: &[DiffFile], tree_visible: bool) -> PagerState {
        let output = render::render(files, 80, false, tree_visible);
        let tree_entries = build_tree_entries(files);
        let mut state = PagerState::new(
            output.lines,
            output.line_map,
            output.file_starts,
            output.hunk_starts,
            tree_entries,
        );
        state.tree_visible = tree_visible;
        state.set_tree_focused(false);
        state
    }

    #[test]
    fn re_render_passes_skip_headers_when_tree_visible() {
        let files = make_two_file_diff();
        let mut state = make_pager_state_from_files(&files, true);
        re_render(&mut state, &files, false, 80);
        // With tree_visible=true, file headers should be skipped
        let stripped: String = state.doc.lines.iter()
            .map(|l| crate::ansi::strip_ansi(l))
            .collect::<Vec<_>>()
            .join("\n");
        assert!(
            !stripped.contains('\u{2500}'),
            "with tree_visible=true, re_render should skip file headers (no box-drawing chars)"
        );
    }

    #[test]
    fn re_render_includes_headers_when_tree_hidden() {
        let files = make_two_file_diff();
        let mut state = make_pager_state_from_files(&files, false);
        re_render(&mut state, &files, false, 80);
        // With tree_visible=false, file headers should be present
        let stripped: String = state.doc.lines.iter()
            .map(|l| crate::ansi::strip_ansi(l))
            .collect::<Vec<_>>()
            .join("\n");
        assert!(
            stripped.contains('\u{2500}'),
            "with tree_visible=false, re_render should include file headers (box-drawing chars)"
        );
    }

    #[test]
    fn re_render_preserves_position_on_header_line() {
        let files = make_two_file_diff();
        // Use tree_visible=false so file headers are emitted, giving us
        // multiple new_lineno=None lines per file_idx to trigger the bug.
        let mut state = make_pager_state_from_files(&files, false);

        // Find a line in file b.txt (file_idx > 0) with new_lineno == None.
        // With tree_visible=false there should be a file header for b.txt and
        // also a deleted line -- the bug is that .position() matches the first
        // None-lineno entry (the file header) even when top_line was on a later one.
        let target = state
            .doc.line_map
            .iter()
            .enumerate()
            .rev() // pick the LAST None-lineno line for file_idx>0
            .find(|(_, li)| li.file_idx > 0 && li.new_lineno.is_none())
            .map(|(i, _)| i)
            .expect("should have a new_lineno=None line with file_idx > 0");

        // Sanity: the first None-lineno line for the same file should be different
        let first_none = state
            .doc.line_map
            .iter()
            .position(|li| li.file_idx == state.doc.line_map[target].file_idx && li.new_lineno.is_none())
            .unwrap();
        assert_ne!(
            first_none, target,
            "need at least two None-lineno lines for the same file to trigger the bug"
        );

        state.top_line = target;
        re_render(&mut state, &files, false, 80);
        assert_eq!(
            state.top_line, target,
            "re_render should preserve top_line on a header/None-lineno line, not jump to {first_none}",
        );
    }

    /// Debug mode toggle must not alter reducer/output state. GD_DEBUG=1 triggers
    /// debug_trace in re_render; this test confirms the resulting state is identical.
    #[test]
    fn debug_toggle_does_not_change_reducer_output() {
        let files = make_two_file_diff();
        let snap_off = with_gd_debug_env(None, || {
            let mut state_off = make_pager_state_from_files(&files, true);
            re_render(&mut state_off, &files, false, 80);
            StateSnapshot::from(&state_off)
        });

        let snap_on = with_gd_debug_env(Some("1"), || {
            let mut state_on = make_pager_state_from_files(&files, true);
            re_render(&mut state_on, &files, false, 80);
            StateSnapshot::from(&state_on)
        });

        assert_eq!(
            snap_off, snap_on,
            "GD_DEBUG on vs off must produce identical state after re_render"
        );
    }

    // ---------------------------------------------------------------------------
    // Chunk-03 regression tests: document swap and remap
    // ---------------------------------------------------------------------------

    /// Regenerate from multi-file to single-file does not panic and preserves valid cursor.
    #[test]
    fn document_swap_multi_to_single_file_preserves_valid_cursor() {
        let raw3 = "\
diff --git a/a.txt b/a.txt
--- a/a.txt
+++ b/a.txt
@@ -1,1 +1,2 @@
 first
+second
diff --git a/b.txt b/b.txt
--- a/b.txt
+++ b/b.txt
@@ -1,2 +1,1 @@
 keep
-remove
diff --git a/c.txt b/c.txt
--- /dev/null
+++ b/c.txt
@@ -0,0 +1,1 @@
+new
";
        let three_files = crate::git::diff::parse(raw3);
        let mut state = make_pager_state_from_files(&three_files, true);
        state.set_active_file(Some(1)); // focus file 1
        state.cursor_line = 35; // somewhere in file 1
        let single_file = crate::git::diff::parse("\
diff --git a/b.txt b/b.txt
--- a/b.txt
+++ b/b.txt
@@ -1,2 +1,1 @@
 keep
-remove
");
        // Simulate regenerate: document swap from 3 files to 1 file
        re_render(&mut state, &single_file, false, 80);
        let (rs, re) = visible_range(&state);
        assert!(
            state.cursor_line >= rs && state.cursor_line < re,
            "cursor {} must be in visible range [{}, {})",
            state.cursor_line,
            rs,
            re
        );
        assert!(state.doc.line_map.get(state.cursor_line).is_some());
    }

    /// Regenerate to empty/no-diff: remap sets cursor/top to 0, no panic.
    #[test]
    fn document_swap_to_empty_exits_cleanly() {
        let files = make_two_file_diff();
        let mut state = make_pager_state_from_files(&files, false);
        state.cursor_line = 5;
        state.top_line = 3;
        let anchor = capture_view_anchor(&state);
        let empty_doc = Document::from_render_output(render::render(&[], 80, false, false));
        remap_after_document_swap(&mut state, anchor, empty_doc, &[]);
        assert_eq!(state.cursor_line, 0);
        assert_eq!(state.top_line, 0);
        assert_eq!(state.visual_anchor, 0);
    }

    /// Resize (re_render at new width) with tree visible keeps valid tree selection and cursor.
    #[test]
    fn resize_with_tree_visible_keeps_valid_selection_and_cursor() {
        let files = make_two_file_diff();
        let mut state = make_pager_state_from_files(&files, true);
        state.set_tree_focused(true);
        state.set_tree_cursor(1); // b.txt
        state.set_active_file(Some(1));
        state.cursor_line = 4; // in b.txt range
        let (tl, tv) = build_tree_lines(&state.tree_entries, state.tree_cursor(), state.tree_width);
        state.tree_lines = tl;
        state.tree_visible_to_entry = tv;
        re_render(&mut state, &files, false, 40); // resize to 40 cols
        assert_state_invariants(&state);
    }

    // ---------------------------------------------------------------------------
    // Mixed-content state builder and navigation tests
    // ---------------------------------------------------------------------------

    /// Build a 90-line PagerState with mixed content (Added/Deleted/Context).
    ///
    /// Same structure as `make_keybinding_state` (3 files, same file_starts and
    /// hunk_starts), but with realistic change groups instead of all-Context.
    ///
    /// File 0 (a.rs, 0-29): headers at 0,5,15. Lines 6-8 Added, 10-11 Deleted.
    /// File 1 (b.rs, 30-59): headers at 30,35,45. Lines 36-37 Added, 46-48 Deleted.
    /// File 2 (c.rs, 60-89): headers at 60,65,75. Lines 66-67 Deleted, 76-78 Added.
    fn make_mixed_content_state() -> PagerState {
        let header_indices: &[usize] = &[0, 5, 15, 30, 35, 45, 60, 65, 75];
        let mut line_map = Vec::with_capacity(90);
        for i in 0..90 {
            let (file_idx, path) = if i < 30 {
                (0, "a.rs")
            } else if i < 60 {
                (1, "b.rs")
            } else {
                (2, "c.rs")
            };
            let line_kind = if header_indices.contains(&i) {
                None
            } else if matches!(i, 6..=8) || matches!(i, 36..=37) || matches!(i, 76..=78) {
                Some(LineKind::Added)
            } else if matches!(i, 10..=11) || matches!(i, 46..=48) || matches!(i, 66..=67) {
                Some(LineKind::Deleted)
            } else {
                Some(LineKind::Context)
            };
            let lineno = if matches!(line_kind, Some(LineKind::Deleted)) {
                // Deleted lines have old_lineno only
                None
            } else if line_kind.is_some() {
                Some(i as u32 + 1)
            } else {
                None
            };
            let old_lineno = if matches!(line_kind, Some(LineKind::Deleted)) {
                Some(i as u32 + 1)
            } else {
                None
            };
            line_map.push(LineInfo {
                file_idx,
                path: path.into(),
                new_lineno: lineno,
                old_lineno,
                line_kind,
            });
        }

        let tree_entries = vec![
            entry("a.rs", 0, Some(0)),
            entry("b.rs", 0, Some(1)),
            entry("c.rs", 0, Some(2)),
        ];
        let mut state = PagerState::new(
            vec!["line".into(); 90],
            line_map,
            vec![0, 30, 60],
            vec![5, 15, 35, 45, 65, 75],
            tree_entries,
        );
        state.cursor_line = 1;
        state.set_tree_focused(false);
        state
    }

    fn add_leading_context_before_hunk_changes(state: &mut PagerState) {
        // Keep hunk_starts anchored on hunk headers, but move first changes deeper
        // into each hunk so d/u must target the first Added/Deleted line.
        state.doc.line_map[6].line_kind = Some(LineKind::Context);
        state.doc.line_map[7].line_kind = Some(LineKind::Context);
        state.doc.line_map[8].line_kind = Some(LineKind::Added);

        state.doc.line_map[16].line_kind = Some(LineKind::Context);
        state.doc.line_map[17].line_kind = Some(LineKind::Deleted);
    }

    // ---------------------------------------------------------------------------
    // Navigation edge cases with mixed content and full_context
    // ---------------------------------------------------------------------------

    #[test]
    fn key_d_hunk_context_skips_leading_context_to_first_change() {
        let mut state = make_mixed_content_state();
        add_leading_context_before_hunk_changes(&mut state);
        state.full_context = false;
        state.cursor_line = 1;
        handle_key(&mut state, Key::Char('d'), 40, 40, &[]);
        assert_eq!(state.cursor_line, 8);
    }

    #[test]
    fn key_u_hunk_context_skips_leading_context_to_prev_first_change() {
        let mut state = make_mixed_content_state();
        add_leading_context_before_hunk_changes(&mut state);
        state.full_context = false;
        state.cursor_line = 17; // first change line in second hunk
        handle_key(&mut state, Key::Char('u'), 40, 40, &[]);
        assert_eq!(state.cursor_line, 8);
    }

    #[test]
    fn key_d_tree_focused_hunk_context_skips_leading_context() {
        let mut state = make_mixed_content_state();
        add_leading_context_before_hunk_changes(&mut state);
        state.full_context = false;
        state.set_tree_focused(true);
        state.cursor_line = 1;
        handle_key(&mut state, Key::Char('d'), 40, 40, &[]);
        assert_eq!(state.cursor_line, 8);
    }

    #[test]
    fn key_d_full_context_single_file_lands_on_change_group() {
        let mut state = make_mixed_content_state();
        state.set_active_file(Some(0));
        state.full_context = true;
        state.cursor_line = 1; // context line before first change group
        handle_key(&mut state, Key::Char('d'), 40, 40, &[]);
        assert_eq!(state.cursor_line, 6);
    }

    #[test]
    fn key_u_full_context_single_file_at_first_change_is_noop() {
        let mut state = make_mixed_content_state();
        state.set_active_file(Some(0));
        state.full_context = true;
        state.cursor_line = 7; // inside first change group (Added 6-8)
        handle_key(&mut state, Key::Char('u'), 40, 40, &[]);
        // Global previous change-group start is line 6.
        assert_eq!(state.cursor_line, 6);
    }

    #[test]
    fn key_d_then_u_round_trip_full_context_single_file() {
        let mut state = make_mixed_content_state();
        state.set_active_file(Some(0));
        state.full_context = true;
        state.cursor_line = 6; // on first change group
        // d should jump to second change group (Deleted at 10)
        handle_key(&mut state, Key::Char('d'), 40, 40, &[]);
        let after_d = state.cursor_line;
        // u should jump back to first change group
        handle_key(&mut state, Key::Char('u'), 40, 40, &[]);
        let after_u = state.cursor_line;
        assert!(after_d > 6, "d should move forward from 6, got {after_d}");
        assert!(after_u <= 8, "u should return near first change group, got {after_u}");
        assert_eq!(after_u, 6);
    }

    #[test]
    fn key_d_full_context_all_context_file_is_noop() {
        // Create a state where file 0 has no changes (all Context)
        let mut state = make_keybinding_state();
        state.set_active_file(Some(0));
        state.full_context = true;
        state.cursor_line = 1;
        // make_keybinding_state has all Context lines → no change groups
        handle_key(&mut state, Key::Char('d'), 40, 40, &[]);
        // Should stay put — no change_group_starts in range
        assert_debug_snapshot!(StateSnapshot::from(&state));
    }

    #[test]
    fn key_d_tree_focused_full_context_single_file() {
        let mut state = make_mixed_content_state();
        state.set_tree_focused(true);
        state.set_active_file(Some(0));
        state.full_context = true;
        state.cursor_line = 1;
        handle_key(&mut state, Key::Char('d'), 40, 40, &[]);
        assert_eq!(state.cursor_line, 6);
    }

    #[test]
    fn key_g_single_file_lands_on_file_start() {
        let mut state = make_mixed_content_state();
        state.set_active_file(Some(1)); // b.rs: lines 30-59
        state.cursor_line = 50;
        handle_key(&mut state, Key::Char('g'), 40, 40, &[]);
        // Should land on first content line of file 1 (31), not global line 1
        assert_debug_snapshot!(StateSnapshot::from(&state));
    }

    #[test]
    #[allow(non_snake_case)]
    fn key_G_single_file_lands_on_file_end() {
        let mut state = make_mixed_content_state();
        state.set_active_file(Some(0)); // a.rs: lines 0-29
        state.cursor_line = 1;
        handle_key(&mut state, Key::Char('G'), 40, 40, &[]);
        // Should land on last content line of file 0 (29), not line 89
        assert_debug_snapshot!(StateSnapshot::from(&state));
    }

    #[test]
    fn key_ctrl_d_single_file_clamps_to_file_end() {
        let mut state = make_mixed_content_state();
        state.set_active_file(Some(0)); // a.rs: lines 0-29
        state.cursor_line = 25;
        handle_key(&mut state, Key::CtrlD, 20, 40, &[]); // half page = 10
        // cursor + 10 = 35, but file 0 ends at 29 → clamped
        assert_debug_snapshot!(StateSnapshot::from(&state));
    }

    #[test]
    fn key_ctrl_u_single_file_clamps_to_file_start() {
        let mut state = make_mixed_content_state();
        state.set_active_file(Some(1)); // b.rs: lines 30-59
        state.cursor_line = 32;
        handle_key(&mut state, Key::CtrlU, 20, 40, &[]); // half page = 10
        // cursor - 10 = 22, but file 1 starts at 30 → clamped
        assert_debug_snapshot!(StateSnapshot::from(&state));
    }

    #[test]
    fn key_j_at_last_content_line_of_single_file_is_noop() {
        let mut state = make_mixed_content_state();
        state.set_active_file(Some(0)); // a.rs: lines 0-29
        state.cursor_line = 29; // last line of file 0
        handle_key(&mut state, Key::Char('j'), 40, 40, &[]);
        // Should stay at 29, not cross to file 1
        assert_debug_snapshot!(StateSnapshot::from(&state));
    }

    #[test]
    #[allow(non_snake_case)]
    fn key_U_no_active_file_at_file_boundary() {
        let mut state = make_mixed_content_state();
        state.set_active_file(None);
        state.cursor_line = 31; // first content line of file 1
        handle_key(&mut state, Key::Char('U'), 50, 40, &[]);
        // Should jump to file 0 start, not get stuck
        assert_debug_snapshot!(StateSnapshot::from(&state));
    }

    // ---------------------------------------------------------------------------
    // Search navigation in single-file mode
    // ---------------------------------------------------------------------------

    #[test]
    fn key_n_wraps_within_single_file() {
        let mut state = make_mixed_content_state();
        state.search_matches = vec![6, 36, 66]; // one match per file
        state.current_match = 0; // at match line 6 (in file 0)
        state.set_active_file(Some(0)); // only file 0 visible
        handle_key(&mut state, Key::Char('n'), 40, 40, &[]);
        // Only match in file 0 is at 6, should wrap back to 6
        assert_debug_snapshot!(StateSnapshot::from(&state));
    }

    #[test]
    #[allow(non_snake_case)]
    fn key_N_wraps_within_single_file() {
        let mut state = make_mixed_content_state();
        state.search_matches = vec![6, 36, 66];
        state.current_match = 0; // at match line 6 (in file 0)
        state.set_active_file(Some(0));
        handle_key(&mut state, Key::Char('N'), 40, 40, &[]);
        // Only match in file 0 is at 6, should wrap back to 6
        assert_debug_snapshot!(StateSnapshot::from(&state));
    }

    #[test]
    fn key_n_no_matches_in_active_file() {
        let mut state = make_mixed_content_state();
        state.search_matches = vec![36, 66]; // no matches in file 0
        state.current_match = -1;
        state.set_active_file(Some(0));
        handle_key(&mut state, Key::Char('n'), 40, 40, &[]);
        // No matches in range 0-29 → no change
        assert_debug_snapshot!(StateSnapshot::from(&state));
    }

    #[test]
    fn key_n_after_toggling_single_file_off_cycles_globally() {
        let mut state = make_mixed_content_state();
        state.search_matches = vec![6, 36, 66];
        state.current_match = 0; // at match 6
        state.set_active_file(None); // all files visible
        handle_key(&mut state, Key::Char('n'), 40, 40, &[]);
        // Should advance to next global match at 36
        assert_debug_snapshot!(StateSnapshot::from(&state));
    }

    // ---------------------------------------------------------------------------
    // Visual mode boundary and yank tests
    // ---------------------------------------------------------------------------

    #[test]
    fn visual_j_clamps_at_file_boundary() {
        let mut state = make_mixed_content_state();
        state.mode = Mode::Visual;
        state.visual_anchor = 28;
        state.cursor_line = 28; // near end of file 0 (file 1 starts at 30)
        // Move j twice: 28→29 should work, 29→30 should be clamped
        handle_key(&mut state, Key::Char('j'), 40, 40, &[]);
        assert_eq!(state.cursor_line, 29, "first j should move to 29");
        handle_key(&mut state, Key::Char('j'), 40, 40, &[]);
        // Line 30 is file 1 (different file_idx) → clamped
        assert_debug_snapshot!(StateSnapshot::from(&state));
    }

    #[test]
    fn visual_k_clamps_at_file_boundary() {
        let mut state = make_mixed_content_state();
        state.mode = Mode::Visual;
        state.visual_anchor = 31;
        state.cursor_line = 31; // first content line of file 1
        // Move k: 31→30 should be clamped (30 is file 1's header, same file_idx)
        // but 30→29 would be file 0 (different file_idx) → clamped there
        handle_key(&mut state, Key::Char('k'), 40, 40, &[]);
        let after_first_k = state.cursor_line;
        handle_key(&mut state, Key::Char('k'), 40, 40, &[]);
        // Can't go below file_start of anchor's file
        assert_debug_snapshot!(StateSnapshot::from(&state));
        // Verify we didn't cross into file 0
        let file_idx = state.doc.line_map[state.cursor_line].file_idx;
        assert_eq!(file_idx, 1, "cursor must remain in file 1, at line {}", state.cursor_line);
        let _ = after_first_k;
    }

    #[test]
    fn visual_y_with_mixed_content_lines() {
        let mut state = make_mixed_content_state();
        state.mode = Mode::Visual;
        state.visual_anchor = 6; // Added line (new_lineno=7)
        state.cursor_line = 11; // Deleted line (old_lineno=12, no new_lineno)
        handle_key(&mut state, Key::Char('y'), 40, 40, &[]);
        // Should produce new_lineno-only references (a.rs:7-9); Deleted lines
        // are skipped for line number resolution since the selection contains Added lines
        assert_debug_snapshot!(StateSnapshot::from(&state));
    }

    #[test]
    fn visual_escape_snaps_cursor_to_content() {
        let mut state = make_keybinding_state();
        state.mode = Mode::Visual;
        state.visual_anchor = 1;
        state.cursor_line = 3;
        state.top_line = 0; // a header line (line_kind None)
        handle_key(&mut state, Key::Escape, 40, 40, &[]);
        assert!(
            is_content_line(&state.doc.line_map, state.cursor_line),
            "cursor_line {} is not a content line",
            state.cursor_line
        );
    }

    #[test]
    fn visual_yank_snaps_cursor_to_content() {
        let mut state = make_keybinding_state();
        state.mode = Mode::Visual;
        state.visual_anchor = 1;
        state.cursor_line = 3;
        state.top_line = 0; // a header line (line_kind None)
        handle_key(&mut state, Key::Char('y'), 40, 40, &[]);
        assert!(
            is_content_line(&state.doc.line_map, state.cursor_line),
            "cursor_line {} is not a content line",
            state.cursor_line
        );
    }

    #[test]
    fn tree_j_snaps_cursor_to_content() {
        let mut state = make_keybinding_state();
        state.set_tree_focused(true);
        state.set_tree_cursor(0); // a.rs
        handle_key(&mut state, Key::Char('j'), 40, 40, &[]);
        // j moves to b.rs; file_starts[1] = 30, which is a header
        assert!(
            is_content_line(&state.doc.line_map, state.cursor_line),
            "cursor_line {} is not a content line",
            state.cursor_line
        );
    }

    #[test]
    fn tree_enter_snaps_cursor_to_content() {
        let mut state = make_keybinding_state();
        state.set_tree_focused(true);
        state.set_tree_cursor(1); // b.rs entry
        handle_key(&mut state, Key::Enter, 40, 40, &[]);
        // Enter scrolls to b.rs; file_starts[1] = 30, which is a header
        assert!(
            is_content_line(&state.doc.line_map, state.cursor_line),
            "cursor_line {} is not a content line",
            state.cursor_line
        );
    }

    #[test]
    fn tree_g_snaps_cursor_to_content() {
        let mut state = make_keybinding_state();
        state.set_tree_focused(true);
        state.cursor_line = 31;
        state.set_tree_cursor(1);
        handle_key(&mut state, Key::Char('g'), 40, 40, &[]);
        // g jumps to first file; file_starts[0] = 0, which is a header
        assert!(
            is_content_line(&state.doc.line_map, state.cursor_line),
            "cursor_line {} is not a content line",
            state.cursor_line
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn key_D_shows_file_status_message() {
        let mut state = make_mixed_content_state();
        state.cursor_line = 1; // inside file 0 (a.rs)
        handle_key(&mut state, Key::Char('D'), 40, 40, &[]);
        // D navigates to next file; status message should say "File" and "b.rs"
        assert_debug_snapshot!(StateSnapshot::from(&state));
    }

    #[test]
    #[allow(non_snake_case)]
    fn key_U_shows_file_status_message() {
        let mut state = make_mixed_content_state();
        state.cursor_line = 31; // inside file 1 (b.rs)
        handle_key(&mut state, Key::Char('U'), 50, 40, &[]);
        // U navigates to previous file; status message should say "File" and "a.rs"
        assert_debug_snapshot!(StateSnapshot::from(&state));
    }

    #[test]
    fn normal_a_toggle_on_snaps_cursor_to_content() {
        let mut state = make_keybinding_state();
        state.cursor_line = 31; // inside b.rs
        handle_key(&mut state, Key::Char('a'), 40, 40, &[]);
        // 'a' activates single-file view; file_starts[1] = 30, which is a header
        assert!(
            is_content_line(&state.doc.line_map, state.cursor_line),
            "cursor_line {} is not a content line",
            state.cursor_line
        );
    }

    // ---------------------------------------------------------------------------
    // Deterministic sequence tests (chunk-06): invariant checks after every step
    // ---------------------------------------------------------------------------

    /// toggle single-file -> tree enter -> context toggle -> regenerate
    #[test]
    fn sequence_toggle_single_file_tree_enter_context_regenerate() {
        let files = vec![make_diff_file("a.rs"), make_diff_file("b.rs"), make_diff_file("c.rs")];
        let mut state = make_keybinding_state();

        handle_key(&mut state, Key::Char('a'), 40, 40, &files);
        assert_state_invariants(&state);

        handle_key(&mut state, Key::Tab, 40, 40, &files);
        assert_state_invariants(&state);

        handle_key(&mut state, Key::Enter, 40, 40, &files);
        assert_state_invariants(&state);

        let result = handle_key(&mut state, Key::Char('z'), 40, 40, &files);
        assert_state_invariants(&state);
        if matches!(result, KeyResult::ReGenerate) {
            re_render(&mut state, &files, false, 80);
        }
        assert_state_invariants(&state);
    }

    /// repeated d/u in both context modes while tree focus changes
    #[test]
    fn sequence_du_in_both_context_modes_with_tree_focus_changes() {
        let mut state = make_mixed_content_state();
        let files: Vec<DiffFile> = vec![];

        state.full_context = false;
        handle_key(&mut state, Key::Char('d'), 40, 40, &files);
        assert_state_invariants(&state);
        handle_key(&mut state, Key::Char('u'), 40, 40, &files);
        assert_state_invariants(&state);

        state.set_tree_focused(true);
        let (tl, tv) = build_tree_lines(&state.tree_entries, state.tree_cursor(), state.tree_width);
        state.tree_lines = tl;
        state.tree_visible_to_entry = tv;

        handle_key(&mut state, Key::Char('d'), 40, 40, &files);
        assert_state_invariants(&state);
        handle_key(&mut state, Key::Char('u'), 40, 40, &files);
        assert_state_invariants(&state);

        state.full_context = true;
        handle_key(&mut state, Key::Char('d'), 40, 40, &files);
        assert_state_invariants(&state);
        handle_key(&mut state, Key::Char('d'), 40, 40, &files);
        assert_state_invariants(&state);
        handle_key(&mut state, Key::Char('u'), 40, 40, &files);
        assert_state_invariants(&state);

        state.set_tree_focused(false);
        handle_key(&mut state, Key::Char('d'), 40, 40, &files);
        assert_state_invariants(&state);
    }

    /// resize + rerender while in search and visual overlays
    #[test]
    fn sequence_resize_rerender_in_search_and_visual_overlays() {
        let files = make_two_file_diff();
        let mut state = make_pager_state_from_files(&files, true);

        handle_key(&mut state, Key::Char('/'), 40, 40, &files);
        state.search_input = "first".to_string();
        state.search_cursor = 5;
        state.search_query = "first".to_string();
        state.search_matches = find_matches(&state.doc.lines, "first");
        state.current_match = find_nearest_match(&state.search_matches, state.top_line);
        state.mode = Mode::Search;
        assert_state_invariants(&state);

        re_render(&mut state, &files, false, 40);
        assert_state_invariants(&state);

        handle_key(&mut state, Key::Escape, 40, 40, &files);
        handle_key(&mut state, Key::Char('v'), 40, 40, &files);
        assert_state_invariants(&state);

        re_render(&mut state, &files, false, 60);
        assert_state_invariants(&state);

        handle_key(&mut state, Key::Escape, 40, 40, &files);
        assert_state_invariants(&state);
    }

    /// Bounded pseudo-random transition test: fixed seed, supported keys,
    /// occasional resize/regenerate. Asserts invariants after every step.
    #[test]
    fn property_bounded_random_transitions() {
        let files = make_two_file_diff();
        let mut state = make_pager_state_from_files(&files, true);

        // Deterministic pseudo-random sequence: LCG with seed 12345
        let mut rng: u64 = 12345;
        let keys: &[Key] = &[
            Key::Char('j'),
            Key::Char('k'),
            Key::Char('d'),
            Key::Char('u'),
            Key::Char('g'),
            Key::Char('G'),
            Key::CtrlD,
            Key::CtrlU,
            Key::Tab,
            Key::Escape,
            Key::Enter,
            Key::Char('a'),
            Key::Char('z'),
            Key::Char('e'),
            Key::Char('v'),
            Key::Char('y'),
            Key::Char('h'),
            Key::Char('l'),
            Key::Char('1'),
            Key::Char('D'),
            Key::Char('U'),
        ];

        for step in 0..72 {
            let key_idx = (rng as usize) % keys.len();
            let key = keys[key_idx];
            rng = rng.wrapping_mul(1103515245).wrapping_add(12345);

            let ch = 24 + ((rng >> 16) as usize % 20);
            let rows = 40;
            let _ = handle_key(&mut state, key, ch, rows, &files);
            assert_state_invariants(&state);

            // Occasional resize (every ~12 steps) or regenerate (every ~18 steps)
            if step > 0 && step % 12 == 0 {
                let cols = 40 + ((rng >> 8) as u16 % 40);
                re_render(&mut state, &files, false, cols);
                assert_state_invariants(&state);
            } else if step > 0 && step % 18 == 0 {
                re_render(&mut state, &files, false, 80);
                assert_state_invariants(&state);
            }
        }
    }

    // ---------------------------------------------------------------------------
    // Reducer-oriented tests (chunk-02): overlay/focus transitions produce valid state
    // ---------------------------------------------------------------------------

    #[test]
    fn reducer_overlay_focus_transitions_produce_valid_state() {
        let files = vec![make_diff_file("a.rs"), make_diff_file("b.rs"), make_diff_file("c.rs")];
        let mut state = make_keybinding_state();

        // Tab: diff -> tree focus
        let _ = handle_key(&mut state, Key::Tab, 40, 40, &files);
        assert!(state.tree_focused());
        assert_state_invariants(&state);

        // Esc: tree -> diff focus
        let _ = handle_key(&mut state, Key::Escape, 40, 40, &files);
        assert!(!state.tree_focused());
        assert_state_invariants(&state);

        // v: enter visual
        let _ = handle_key(&mut state, Key::Char('v'), 40, 40, &files);
        assert_eq!(state.mode, Mode::Visual);
        assert_state_invariants(&state);

        // y: visual yank -> normal
        let _ = handle_key(&mut state, Key::Char('y'), 40, 40, &files);
        assert_eq!(state.mode, Mode::Normal);
        assert_state_invariants(&state);

        // e: open tree, then e: close tree
        let _ = handle_key(&mut state, Key::Char('e'), 40, 40, &files);
        state.set_active_file(Some(1));
        let _ = handle_key(&mut state, Key::Char('e'), 40, 40, &files);
        assert!(!state.tree_visible);
        assert_state_invariants(&state);
    }
}
