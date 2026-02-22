use crate::git::diff::{DiffFile, FileStatus};
use crate::style;

#[derive(Debug)]
pub(crate) struct TreeEntry {
    pub(crate) label: String,
    pub(crate) depth: usize,
    pub(crate) file_idx: Option<usize>,
    pub(crate) status: Option<FileStatus>,
    pub(crate) collapsed: bool,
}

pub(crate) fn build_tree_entries(files: &[DiffFile]) -> Vec<TreeEntry> {
    let mut indexed: Vec<(usize, &str)> = files.iter().enumerate().map(|(i, f)| (i, f.path())).collect();
    indexed.sort_by(|a, b| a.1.cmp(b.1));

    let mut entries: Vec<TreeEntry> = Vec::new();
    let mut prev_components: Vec<&str> = Vec::new();

    for (file_idx, path) in &indexed {
        let parts: Vec<&str> = path.split('/').collect();
        let dir_parts = &parts[..parts.len().saturating_sub(1)];
        let basename = parts[parts.len().saturating_sub(1)];

        let common = prev_components
            .iter()
            .zip(dir_parts.iter())
            .take_while(|(a, b)| a == b)
            .count();

        for (depth, &component) in dir_parts.iter().enumerate().skip(common) {
            entries.push(TreeEntry {
                label: component.to_string(),
                depth,
                file_idx: None,
                status: None,
                collapsed: false,
            });
        }

        entries.push(TreeEntry {
            label: basename.to_string(),
            depth: dir_parts.len(),
            file_idx: Some(*file_idx),
            status: Some(files[*file_idx].status),
            collapsed: false,
        });

        prev_components = dir_parts.to_vec();
    }

    let mut i = 0;
    while i + 1 < entries.len() {
        if entries[i].file_idx.is_none()
            && entries[i + 1].file_idx.is_none()
            && entries[i + 1].depth == entries[i].depth + 1
        {
            let parent_depth = entries[i].depth;
            let has_sibling = entries[i + 2..]
                .iter()
                .any(|e| e.depth <= parent_depth + 1 && e.depth > parent_depth)
                && entries[i + 2..]
                    .iter()
                    .take_while(|e| e.depth > parent_depth)
                    .any(|e| e.depth == parent_depth + 1);
            if !has_sibling {
                let child_label = entries[i + 1].label.clone();
                entries[i].label = format!("{}/{}", entries[i].label, child_label);
                let removed_depth = entries[i + 1].depth;
                entries.remove(i + 1);
                for e in &mut entries[i + 1..] {
                    if e.depth > removed_depth {
                        e.depth -= 1;
                    } else {
                        break;
                    }
                }
                continue;
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
            let status_extra = if e.file_idx.is_some() && e.status.is_some() {
                2
            } else {
                0
            };
            (e.depth + 1) * 4 + 2 + status_extra + e.label.len() + 2
        })
        .max()
        .unwrap_or(0);
    max_len.min(40)
}

pub(crate) fn file_idx_to_entry_idx(tree_entries: &[TreeEntry], file_idx: usize) -> usize {
    tree_entries
        .iter()
        .position(|e| e.file_idx == Some(file_idx))
        .unwrap_or(0)
}

pub(crate) fn compute_connector_prefix(visible: &[&TreeEntry], idx: usize) -> String {
    let depth = visible[idx].depth;
    let mut prefix = String::new();

    for d in 0..depth {
        let has_continuation = visible[idx + 1..].iter().any(|e| e.depth <= d);
        if has_continuation {
            prefix.push_str("│   ");
        } else {
            prefix.push_str("    ");
        }
    }

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
pub(crate) fn build_tree_lines(
    tree_entries: &[TreeEntry],
    cursor_entry_idx: usize,
    width: usize,
) -> (Vec<String>, Vec<usize>) {
    let mut visible: Vec<&TreeEntry> = Vec::new();
    let mut visible_orig: Vec<usize> = Vec::new();
    let mut collapse_depth: Option<usize> = None;

    for (i, entry) in tree_entries.iter().enumerate() {
        if let Some(cd) = collapse_depth {
            if entry.depth > cd {
                continue;
            }
            collapse_depth = None;
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

        let status_extra = if entry.file_idx.is_some() && entry.status.is_some() {
            2
        } else {
            0
        };
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
