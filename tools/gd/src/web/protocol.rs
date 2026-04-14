use serde::{Deserialize, Serialize};

use crate::git::diff::{FileStatus, LineKind};

#[derive(Serialize)]
#[serde(tag = "type")]
pub(crate) enum ServerMessage {
    DiffData {
        files: Vec<WebDiffFile>,
        tree: Vec<WebTreeEntry>,
    },
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
pub(crate) enum ClientMessage {
    SetFullContext { enabled: bool },
}

#[derive(Serialize)]
pub(crate) struct WebDiffFile {
    pub path: String,
    pub old_path: Option<String>,
    pub status: WebFileStatus,
    pub hunks: Vec<WebDiffHunk>,
}

#[derive(Serialize)]
pub(crate) struct WebDiffHunk {
    pub old_start: u32,
    pub new_start: u32,
    pub lines: Vec<WebDiffLine>,
}

#[derive(Serialize)]
pub(crate) struct WebDiffLine {
    pub kind: WebLineKind,
    pub content_html: String,
    pub raw_content: String,
    pub old_lineno: Option<u32>,
    pub new_lineno: Option<u32>,
}

#[derive(Serialize)]
pub(crate) struct WebTreeEntry {
    pub label: String,
    pub depth: usize,
    pub file_idx: Option<usize>,
    pub status: Option<WebFileStatus>,
    pub is_dir: bool,
    pub collapsed: bool,
    pub icon: String,
    pub icon_color: String,
}

#[derive(Serialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub(crate) enum WebFileStatus {
    Modified,
    Added,
    Deleted,
    Renamed,
    Untracked,
}

impl From<FileStatus> for WebFileStatus {
    fn from(s: FileStatus) -> Self {
        match s {
            FileStatus::Modified => Self::Modified,
            FileStatus::Added => Self::Added,
            FileStatus::Deleted => Self::Deleted,
            FileStatus::Renamed => Self::Renamed,
            FileStatus::Untracked => Self::Untracked,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum WebLineKind {
    Context,
    Added,
    Deleted,
}

impl From<LineKind> for WebLineKind {
    fn from(k: LineKind) -> Self {
        match k {
            LineKind::Context => Self::Context,
            LineKind::Added => Self::Added,
            LineKind::Deleted => Self::Deleted,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_message_deserialize_set_full_context() {
        let json = r#"{"type":"SetFullContext","enabled":true}"#;
        let msg: ClientMessage = serde_json::from_str(json).unwrap();
        assert!(matches!(msg, ClientMessage::SetFullContext { enabled: true }));
    }
}
