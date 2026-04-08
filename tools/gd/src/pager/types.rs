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

/// Which pane has keyboard focus.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum FocusPane {
    #[default]
    Diff,
    Tree,
}

/// View scope: all files or single file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ViewScope {
    AllFiles,
    SingleFile(FileIx),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    ApplyPatch { patch: String, cached: bool, reverse: bool },
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
    CopyRelPath,
    CopyAbsPath,
    OpenEditor,
    Reload,
    ToggleTooltip,
    StageLine,
    StageHunk,
    DiscardLine,
    DiscardHunk,
    ToggleFocus,
    FocusDiff,
    FocusTree,
    TreeEnter,
}

/// Help group for overlay layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum HelpGroup {
    Navigation,
    DiffNav,
    Search,
    Selection,
    Staging,
    Tree,
    Other,
}
