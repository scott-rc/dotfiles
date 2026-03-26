use std::collections::HashSet;

use crate::git::diff::{DiffFile, FileStatus};
use crate::style;

#[derive(Debug)]
pub struct TreeEntry {
    pub label: String,
    pub depth: usize,
    pub file_idx: Option<usize>,
    pub status: Option<FileStatus>,
    pub collapsed: bool,
}

pub fn build_tree_entries(files: &[DiffFile]) -> Vec<TreeEntry> {
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

    // Collapse single-child directory chains (a/b/c -> a/b/c).
    // O(n^2) for deep nesting; acceptable for real codebases.
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

    // Default single-child chains (label contains `/`) to collapsed.
    for e in &mut entries {
        if e.file_idx.is_none() && e.label.contains('/') {
            e.collapsed = true;
        }
    }

    entries
}

pub(crate) const MIN_DIFF_WIDTH: usize = 80;
pub(crate) const MIN_TREE_WIDTH: usize = 15;
const MAX_TREE_WIDTH: usize = 40;

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
    let allocated = content_width.min(MAX_TREE_WIDTH).min(terminal_cols.saturating_sub(MIN_DIFF_WIDTH + 1));
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

/// Reconstruct the full `/`-joined path of a tree entry by walking backward
/// from `idx` to find parent directories at decreasing depth.
pub fn tree_entry_path(entries: &[TreeEntry], idx: usize) -> String {
    let entry = &entries[idx];
    if entry.depth == 0 {
        return entry.label.clone();
    }
    let mut parts = vec![entry.label.as_str()];
    let mut target_depth = entry.depth - 1;
    for i in (0..idx).rev() {
        if entries[i].depth == target_depth && entries[i].file_idx.is_none() {
            parts.push(entries[i].label.as_str());
            if target_depth == 0 {
                break;
            }
            target_depth -= 1;
        }
    }
    parts.reverse();
    parts.join("/")
}

/// Re-apply collapse state from `collapsed_paths` to tree entries after a rebuild.
pub(crate) fn apply_collapse_state(entries: &mut [TreeEntry], collapsed_paths: &HashSet<String>) {
    for i in 0..entries.len() {
        if entries[i].file_idx.is_none() {
            entries[i].collapsed = collapsed_paths.contains(&tree_entry_path(entries, i));
        }
    }
}

#[cfg(test)]
pub(crate) fn compute_connector_prefix(
    visible: &[&TreeEntry],
    idx: usize,
    start_depth: usize,
) -> String {
    let depth = visible[idx].depth;
    let mut prefix = String::new();

    for d in start_depth..depth {
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

/// Precompute connector prefixes for all visible entries in a single pass.
///
/// For each entry, we need two pieces of information at each ancestor depth:
/// 1. Whether any later entry reaches that depth or shallower (continuation line).
/// 2. Whether the entry is the last sibling at its own depth (└── vs ├──).
///
/// We do a backward pass to build `next_at_depth[d]` = index of the next entry
/// at depth <= d, looking forward from each position. Then connector strings
/// are assembled in a forward pass using these precomputed lookups.
pub(crate) fn precompute_connectors(
    visible: &[&TreeEntry],
    start_depth: usize,
) -> Vec<String> {
    let n = visible.len();
    if n == 0 {
        return Vec::new();
    }

    let max_depth = visible.iter().map(|e| e.depth).max().unwrap_or(0);

    // For each depth d, track whether we've seen an entry at depth <= d
    // scanning backward. `has_later_at_depth[d]` is true if, from the current
    // position forward, there exists an entry with depth <= d.
    // We store per-entry: for each depth d, does a later entry have depth <= d?
    // And: does a later sibling at the same depth exist?

    // Backward pass: for each entry i, compute:
    //   has_continuation[i][d] = exists j > i where visible[j].depth <= d
    //   has_sibling_after[i] = exists j > i where visible[j].depth == depth_i
    //                          AND all entries between i+1..j have depth >= depth_i

    // Efficient approach: track the minimum depth seen so far (scanning backward).
    // For has_continuation at depth d: min_depth_after[i] <= d.
    // For has_sibling_after: need the next entry at same depth with no shallower
    // entry in between.

    // min_depth_after[i] = min depth among visible[i+1..]
    let mut min_depth_after = vec![usize::MAX; n];
    {
        let mut min_so_far = usize::MAX;
        for i in (0..n - 1).rev() {
            min_so_far = min_so_far.min(visible[i + 1].depth);
            min_depth_after[i] = min_so_far;
        }
    }

    // For has_sibling_after: for each entry i at depth d, check if among
    // entries i+1.. there is one at depth d before any at depth < d.
    // We can precompute this with a backward pass per depth, but that's
    // O(max_depth * n). Instead, do a single backward pass tracking the
    // next index at each depth using a stack-like approach.
    //
    // next_same_depth_before_shallower[i] = true if the next entry at depth <= d
    // (where d = visible[i].depth) has depth == d (i.e., it's a sibling, not a
    // shallower ancestor).
    let mut has_sibling = vec![false; n];
    {
        // last_at_depth[d] = most recent index (scanning backward) where depth == d
        // and no shallower entry appeared between that index and the current scan position.
        // When we encounter depth d, we clear all last_at_depth entries for depths > d
        // (they're no longer valid siblings — a shallower entry intervened).
        let mut last_at_depth: Vec<Option<usize>> = vec![None; max_depth + 1];
        for i in (0..n).rev() {
            let d = visible[i].depth;
            // Check if there's a later entry at the same depth with no shallower in between.
            if last_at_depth[d].is_some() {
                has_sibling[i] = true;
            }
            // Record this entry at its depth.
            last_at_depth[d] = Some(i);
            // Invalidate deeper depths: a shallower entry breaks sibling chains.
            for slot in last_at_depth.iter_mut().skip(d + 1) {
                *slot = None;
            }
        }
    }

    // Build prefixes
    let mut result = Vec::with_capacity(n);
    for i in 0..n {
        let depth = visible[i].depth;
        let mut prefix = String::new();

        for d in start_depth..depth {
            if min_depth_after[i] <= d {
                prefix.push_str("│   ");
            } else {
                prefix.push_str("    ");
            }
        }

        if has_sibling[i] {
            prefix.push_str("├── ");
        } else {
            prefix.push_str("└── ");
        }

        result.push(prefix);
    }

    result
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
    focused: bool,
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

    // Compute indent_offset: how many depth levels to scroll off-screen
    // so the cursor entry's label fits in the panel.
    let indent_offset = {
        let ce = visible[cursor_vis];
        let ce_status = if ce.file_idx.is_some() && ce.status.is_some() {
            2
        } else {
            0
        };
        let ce_prefix = (ce.depth + 1) * 4 + 2 + ce_status;
        let ce_budget = width.saturating_sub(ce_prefix);
        let ce_label_len = ce.label.chars().count();
        if ce_label_len > ce_budget {
            let extra = ce_label_len - ce_budget;
            extra.div_ceil(4).min(ce.depth) // round up to next indent level
        } else {
            0
        }
    };

    let connectors = precompute_connectors(&visible, indent_offset);
    let mut lines = Vec::new();

    for (vi, &entry) in visible.iter().enumerate() {
        let orig_idx = visible_orig[vi];

        let use_indicator = indent_offset > 0 && entry.depth < indent_offset;
        let prefix = if use_indicator {
            "..".to_string()
        } else {
            connectors[vi].clone()
        };

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

        let prefix_width = if use_indicator {
            2 + 2 + status_extra // `..` + icon + space + optional status
        } else {
            (entry.depth - indent_offset + 1) * 4 + 2 + status_extra
        };
        let label_budget = width.saturating_sub(prefix_width);
        let label = truncate_label(&entry.label, label_budget);
        let label_len = label.chars().count();

        let vis_len = prefix_width + label_len;
        let right_pad = width.saturating_sub(vis_len);
        let guide = style::FG_TREE_GUIDE;

        if orig_idx == cursor_entry_idx {
            let reset = style::RESET;
            let (bg, fg) = if focused {
                (style::BG_TREE_CURSOR_FOCUSED, style::FG_TREE_CURSOR_FOCUSED)
            } else {
                (style::BG_TREE_CURSOR_UNFOCUSED, style::FG_FILE_HEADER)
            };
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
