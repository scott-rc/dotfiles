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
    Search,
}

/// Action identifiers. Single source of truth for key->action mapping.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ActionId {
    Quit,
    ScrollDown,
    ScrollUp,
    HalfPageDown,
    HalfPageUp,
    Top,
    Bottom,
    CenterViewport,
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
    VisualSelect,
    YankSelection,
    OpenEditor,
    ToggleTooltip,
}

/// Help group for overlay layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum HelpGroup {
    Navigation,
    DiffNav,
    Search,
    Selection,
    Other,
}
