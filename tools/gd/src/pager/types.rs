#![allow(dead_code)]

/// Valid file index into `file_starts`. Construct via `FileIx::new(idx, file_count)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct FileIx(pub(crate) usize);

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
pub(crate) struct LineIx(pub(crate) usize);

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
pub(crate) struct TreeEntryIx(pub(crate) usize);

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

