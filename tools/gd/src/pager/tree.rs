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
    debug_assert!(
        files.windows(2).all(|w| w[0].path() <= w[1].path()),
        "build_tree_entries expects files sorted by path"
    );
    let indexed: Vec<(usize, &str)> = files
        .iter()
        .enumerate()
        .map(|(i, f)| (i, f.path()))
        .collect();

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

pub(crate) const MIN_DIFF_WIDTH: usize = 80;
pub(crate) const MIN_TREE_WIDTH: usize = 15;
const FISHEYE_RADIUS: usize = 2;

pub(crate) fn compute_tree_width(tree_entries: &[TreeEntry]) -> usize {
    tree_entries
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
        .unwrap_or(0)
}

pub(crate) fn resolve_tree_layout(
    content_width: usize,
    terminal_cols: usize,
    has_directories: bool,
    file_count: usize,
) -> Option<usize> {
    if !has_directories && file_count < 4 {
        return None;
    }
    let allocated = content_width.min(terminal_cols.saturating_sub(MIN_DIFF_WIDTH + 1));
    if allocated < MIN_TREE_WIDTH {
        return None;
    }
    Some(allocated)
}

pub(crate) fn truncate_label(label: &str, max_width: usize) -> String {
    if label.chars().count() <= max_width {
        return label.to_string();
    }
    if max_width == 0 {
        return String::new();
    }
    if max_width == 1 {
        return ".".to_string();
    }
    if max_width == 2 {
        return "..".to_string();
    }
    let keep = max_width.saturating_sub(2);
    let truncated: String = label.chars().take(keep).collect();
    format!("{truncated}..")
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

    let cursor_vis = visible_orig
        .iter()
        .position(|&oi| oi == cursor_entry_idx)
        .unwrap_or(0);

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

        let prefix_width = (entry.depth + 1) * 4 + 2 + status_extra;
        let label_budget = width.saturating_sub(prefix_width);
        let expanded = vi.abs_diff(cursor_vis) <= FISHEYE_RADIUS;
        let label = if expanded {
            truncate_label(&entry.label, label_budget)
        } else {
            truncate_label(&entry.label, label_budget * 2 / 3)
        };
        let label_len = label.chars().count();

        let vis_len = prefix_width + label_len;
        let right_pad = width.saturating_sub(vis_len);
        let guide = style::FG_TREE_GUIDE;

        if orig_idx == cursor_entry_idx {
            let reset = style::RESET;
            let fg = style::FG_FILE_HEADER;
            let bg = style::BG_TREE_CURSOR;
            let rpad = " ".repeat(right_pad);
            if entry.file_idx.is_some() {
                if let Some(st) = entry.status {
                    let (sc, sc_color) = status_symbol(st);
                    lines.push(format!("{bg}{guide}{prefix}{reset}{bg}{icon_color}{icon} {sc_color}{sc}{fg} {label}{rpad}{reset}"));
                } else {
                    lines.push(format!(
                        "{bg}{guide}{prefix}{reset}{bg}{icon_color}{icon} {fg}{label}{rpad}{reset}"
                    ));
                }
            } else {
                lines.push(format!(
                    "{bg}{guide}{prefix}{reset}{bg}{icon_color}{icon} {fg}{label}{rpad}{reset}"
                ));
            }
        } else if entry.file_idx.is_some() {
            let reset = style::RESET;
            let fg = style::FG_TREE;
            let rpad = " ".repeat(right_pad);
            if let Some(st) = entry.status {
                let (sc, sc_color) = status_symbol(st);
                lines.push(format!("{guide}{prefix}{reset}{icon_color}{icon}{reset} {sc_color}{sc}{reset} {fg}{label}{rpad}{reset}"));
            } else {
                lines.push(format!(
                    "{guide}{prefix}{reset}{icon_color}{icon}{reset} {fg}{label}{rpad}{reset}"
                ));
            }
        } else {
            let reset = style::RESET;
            let fg = style::FG_TREE_DIR;
            let rpad = " ".repeat(right_pad);
            lines.push(format!(
                "{guide}{prefix}{reset}{icon_color}{icon}{reset} {fg}{label}{rpad}{reset}"
            ));
        }
    }

    (lines, visible_orig)
}
