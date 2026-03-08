use tui::pager::{Key, copy_to_clipboard};

use std::collections::HashSet;

use crate::git::diff::DiffFile;
use crate::git::DiffSource;

use super::content::{is_content_line, next_content_line, prev_content_line};
use super::keymap::keymap_lookup;
use super::navigation::{
    nav_D_down, nav_U_up, nav_du_down, nav_du_up, recenter_top_line, sync_active_file_to_cursor,
    jump_to_tree_file, sync_tree_cursor, tree_cursor_bottom, tree_cursor_down, tree_cursor_top,
    tree_cursor_up, viewport_bounds,
};
use super::rendering::{enforce_scrolloff, format_copy_ref, resolve_lineno};
use super::search::{
    cancel_search, handle_search_key, next_match_in_range, prev_match_in_range, scroll_to_match,
    submit_search,
};
use super::state::{PagerState, ReducerCtx, ReducerEffect};
use super::state::{clamp_cursor_and_top, debug_assert_valid_state, visible_range};
use super::tree::{build_tree_entries, compute_tree_width, file_idx_to_entry_idx, resolve_tree_layout, tree_entry_path, MIN_DIFF_WIDTH};
use super::types::{ActionId, FocusPane, KeyContext, KeyResult, Mode};

#[derive(Debug, Clone)]
pub(crate) enum ReducerEvent {
    Key(Key),
}

fn set_view_to_file(state: &mut PagerState, file_idx: usize, ch: usize) {
    // Save current position before switching away
    if let Some(current_idx) = state.active_file() {
        state.file_positions.insert(current_idx, (state.top_line, state.cursor_line));
    }

    state.set_active_file(Some(file_idx));
    if let Some(start) = state.file_start(file_idx) {
        // Restore cached position if available
        if let Some(&(saved_top, saved_cursor)) = state.file_positions.get(&file_idx) {
            let end = state.file_end(file_idx);
            let max = end.saturating_sub(1);
            state.cursor_line = saved_cursor.clamp(start, max);
            let (_, _, _, max_top) = viewport_bounds(state, ch);
            state.top_line = saved_top.clamp(start, max_top);
        } else {
            state.top_line = start;
            // Keep cursor at file header so ] can find the first change group.
            // snap_to_content would land on a change line when there's no leading
            // context, making jump_next (strictly >) unable to find that target.
            state.cursor_line = start;
            let (_, _, _, max_top) = viewport_bounds(state, ch);
            state.top_line = recenter_top_line(state.cursor_line, ch, start, max_top);
        }
    }
}

fn dispatch_normal_action(
    state: &mut PagerState,
    action: ActionId,
    ctx: &ReducerCtx<'_>,
) -> Option<ReducerEffect> {
    let ch = ctx.content_height;
    let files = ctx.files;
    let half_page = ch / 2;
    let (range_start, range_end) = visible_range(state);
    let max_cursor = range_end.saturating_sub(1);

    match action {
        ActionId::Quit => Some(ReducerEffect::Quit),
        ActionId::ScrollDown => {
            let next = (state.cursor_line + 1).min(max_cursor);
            state.cursor_line = next_content_line(&state.doc.line_map, next, max_cursor);
            None
        }
        ActionId::ScrollUp => {
            let prev = state.cursor_line.saturating_sub(1).max(range_start);
            state.cursor_line = prev_content_line(&state.doc.line_map, prev, range_start);
            None
        }
        ActionId::HalfPageDown => {
            let target = (state.cursor_line + half_page).min(max_cursor);
            state.cursor_line = next_content_line(&state.doc.line_map, target, max_cursor);
            if !is_content_line(&state.doc.line_map, state.cursor_line) {
                state.cursor_line = prev_content_line(&state.doc.line_map, target, range_start);
            }
            None
        }
        ActionId::HalfPageUp => {
            let target = state.cursor_line.saturating_sub(half_page).max(range_start);
            state.cursor_line = prev_content_line(&state.doc.line_map, target, range_start);
            if !is_content_line(&state.doc.line_map, state.cursor_line) {
                state.cursor_line = next_content_line(&state.doc.line_map, target, max_cursor);
            }
            None
        }
        ActionId::Top => {
            state.cursor_line = next_content_line(&state.doc.line_map, range_start, max_cursor);
            None
        }
        ActionId::Bottom => {
            state.cursor_line = prev_content_line(&state.doc.line_map, max_cursor, range_start);
            None
        }
        ActionId::CenterViewport => {
            let (rs, _, _, max_top) = viewport_bounds(state, ch);
            state.top_line = recenter_top_line(state.cursor_line, ch, rs, max_top);
            None
        }
        ActionId::NextHunk => {
            let res = nav_du_down(state);
            state.cursor_line = res.cursor_line;
            state.status_message.clone_from(&res.status_message);
            sync_active_file_to_cursor(state);
            if res.moved {
                let (rs, _, _, max_top) = viewport_bounds(state, ch);
                state.top_line = recenter_top_line(state.cursor_line, ch, rs, max_top);
            } else if let Some(active) = state.active_file() {
                // At last hunk of current file -- advance to next file
                if active + 1 < state.file_count() {
                    set_view_to_file(state, active + 1, ch);
                    // Jump to the first change group in the new file
                    let fwd = nav_du_down(state);
                    if fwd.moved {
                        state.cursor_line = fwd.cursor_line;
                        state.status_message.clone_from(&fwd.status_message);
                        let (rs, _, _, max_top) = viewport_bounds(state, ch);
                        state.top_line = recenter_top_line(state.cursor_line, ch, rs, max_top);
                    } else {
                        state.status_message = "Next file".into();
                    }
                }
            }
            sync_tree_cursor(state, ch);
            Some(ReducerEffect::Continue)
        }
        ActionId::PrevHunk => {
            let res = nav_du_up(state);
            state.cursor_line = res.cursor_line;
            state.status_message.clone_from(&res.status_message);
            sync_active_file_to_cursor(state);
            if res.moved {
                let (rs, _, _, max_top) = viewport_bounds(state, ch);
                state.top_line = recenter_top_line(state.cursor_line, ch, rs, max_top);
            } else if let Some(active) = state.active_file() {
                // At first hunk of current file -- retreat to previous file
                if active > 0 {
                    set_view_to_file(state, active - 1, ch);
                    // Jump to the last change group in the new file
                    let targets = super::navigation::du_nav_targets(state);
                    if let Some(&last_target) = targets.last() {
                        let max_line = state.doc.line_map.len().saturating_sub(1);
                        state.cursor_line = super::content::next_content_line(
                            &state.doc.line_map, last_target, max_line,
                        );
                        state.status_message = super::navigation::nav_status_message(
                            if state.full_context { "Change" } else { "Hunk" },
                            state.cursor_line,
                            &targets,
                            &state.doc.line_map,
                        );
                        let (rs, _, _, max_top) = viewport_bounds(state, ch);
                        state.top_line = recenter_top_line(state.cursor_line, ch, rs, max_top);
                    } else {
                        state.status_message = "Previous file".into();
                    }
                }
            }
            sync_tree_cursor(state, ch);
            Some(ReducerEffect::Continue)
        }
        ActionId::NextFile => {
            if let Some(active) = state.active_file() {
                if active + 1 < state.file_count() {
                    set_view_to_file(state, active + 1, ch);
                    state.status_message = "Next file".into();
                }
            } else {
                let res = nav_D_down(state, ch);
                state.cursor_line = res.cursor_line;
                state.top_line = res.top_line;
                state.status_message.clone_from(&res.status_message);
            }
            sync_tree_cursor(state, ch);
            Some(ReducerEffect::Continue)
        }
        ActionId::PrevFile => {
            if let Some(active) = state.active_file() {
                if active > 0 {
                    set_view_to_file(state, active - 1, ch);
                    state.status_message = "Previous file".into();
                }
            } else {
                let res = nav_U_up(state, ch);
                state.cursor_line = res.cursor_line;
                state.top_line = res.top_line;
                state.status_message.clone_from(&res.status_message);
            }
            sync_tree_cursor(state, ch);
            Some(ReducerEffect::Continue)
        }
        ActionId::ToggleSingleFile => {
            state.view_scope_user_set = true;
            if state.active_file().is_some() {
                state.set_active_file(None);
                state.status_message = "All files".into();
            } else {
                let anchor = state.cursor_line;
                let file_idx = state.doc.line_map.get(anchor).map_or(0, |li| li.file_idx);
                if state.tree_entries.is_empty() {
                    state.tree_entries = build_tree_entries(files);
                    let content_width = compute_tree_width(&state.tree_entries);
                    let has_directories = state.tree_entries.iter().any(|e| e.file_idx.is_none());
                    let fc = state.doc.file_count();
                    let terminal_cols = ctx.cols as usize;
                    state.tree_width = resolve_tree_layout(content_width, terminal_cols, has_directories, fc)
                        .unwrap_or_else(|| terminal_cols.saturating_sub(MIN_DIFF_WIDTH + 1));
                }
                set_view_to_file(state, file_idx, ch);
                state.set_tree_cursor(file_idx_to_entry_idx(&state.tree_entries, file_idx));
                state.rebuild_tree_lines();
                state.status_message = "Single file".into();
            }
            Some(ReducerEffect::ReRender)
        }
        ActionId::ToggleFullContext => {
            state.full_context_user_set = true;
            state.full_context = !state.full_context;
            state.status_message = if state.full_context {
                "Full file context".into()
            } else {
                "Hunk context".into()
            };
            Some(ReducerEffect::ReGenerate)
        }
        ActionId::Search => {
            state.mode = Mode::Search;
            None
        }
        ActionId::NextMatch => {
            if !state.search_matches.is_empty() {
                if state.active_file().is_some() {
                    let (rs, re) = visible_range(state);
                    if let Some(idx) =
                        next_match_in_range(&state.search_matches, state.current_match, rs, re)
                    {
                        state.current_match = idx;
                        scroll_to_match(state, ch);
                    }
                } else {
                    let len = state.search_matches.len() as isize;
                    if state.current_match < 0 || state.current_match >= len {
                        state.current_match = 0;
                    } else {
                        state.current_match = (state.current_match + 1) % len;
                    }
                    scroll_to_match(state, ch);
                }
            }
            None
        }
        ActionId::PrevMatch => {
            if !state.search_matches.is_empty() {
                if state.active_file().is_some() {
                    let (rs, re) = visible_range(state);
                    if let Some(idx) =
                        prev_match_in_range(&state.search_matches, state.current_match, rs, re)
                    {
                        state.current_match = idx;
                        scroll_to_match(state, ch);
                    }
                } else {
                    let len = state.search_matches.len() as isize;
                    if state.current_match < 0 || state.current_match >= len {
                        state.current_match = len - 1;
                    } else {
                        state.current_match = (state.current_match - 1 + len) % len;
                    }
                    scroll_to_match(state, ch);
                }
            }
            None
        }
        ActionId::ToggleTree => {
            state.tree_visible = !state.tree_visible;
            state.tree_user_hidden = !state.tree_visible;
            if state.tree_visible {
                let anchor = state.cursor_line;
                let file_idx = state.doc.line_map.get(anchor).map_or(0, |li| li.file_idx);
                state.tree_entries = build_tree_entries(files);
                let content_width = compute_tree_width(&state.tree_entries);
                let has_directories = state.tree_entries.iter().any(|e| e.file_idx.is_none());
                let file_count = state.doc.file_count();
                let terminal_cols = ctx.cols as usize;
                state.tree_width = resolve_tree_layout(content_width, terminal_cols, has_directories, file_count)
                    .unwrap_or_else(|| terminal_cols.saturating_sub(MIN_DIFF_WIDTH + 1));
                state.set_tree_cursor(file_idx_to_entry_idx(&state.tree_entries, file_idx));
                state.rebuild_tree_lines();
            }
            Some(ReducerEffect::ReRender)
        }
        ActionId::VisualSelect => {
            state.visual_anchor = Some(state.cursor_line);
            state.status_message = "-- VISUAL --".to_string();
            Some(ReducerEffect::Continue)
        }
        ActionId::YankSelection => {
            if let Some(anchor) = state.visual_anchor {
                let lo = anchor.min(state.cursor_line);
                let hi = anchor.max(state.cursor_line);
                let path = state
                    .doc
                    .line_map
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
                state.visual_anchor = None;
            } else {
                state.status_message = "No selection".to_string();
            }
            Some(ReducerEffect::Continue)
        }
        ActionId::CopyRelPath => {
            let pos = state
                .cursor_line
                .min(state.doc.line_map.len().saturating_sub(1));
            if !state.doc.line_map.is_empty() {
                let path = state.doc.line_map[pos].path.clone();
                let ok = copy_to_clipboard(&path);
                state.status_message = if ok {
                    format!("Copied: {path}")
                } else {
                    "Copy failed (pbcopy not available)".to_string()
                };
            }
            Some(ReducerEffect::Continue)
        }
        ActionId::CopyAbsPath => {
            let pos = state
                .cursor_line
                .min(state.doc.line_map.len().saturating_sub(1));
            if !state.doc.line_map.is_empty() {
                let rel = &*state.doc.line_map[pos].path;
                let abs = ctx.repo.join(rel);
                let text = abs.to_string_lossy().to_string();
                let ok = copy_to_clipboard(&text);
                state.status_message = if ok {
                    format!("Copied: {text}")
                } else {
                    "Copy failed (pbcopy not available)".to_string()
                };
            }
            Some(ReducerEffect::Continue)
        }
        ActionId::OpenEditor => {
            let pos = state
                .cursor_line
                .min(state.doc.line_map.len().saturating_sub(1));
            if !state.doc.line_map.is_empty() {
                let info = &state.doc.line_map[pos];
                let path = info.path.to_string();
                let lineno = info.new_lineno;
                return Some(ReducerEffect::OpenEditor { path, lineno });
            }
            None
        }
        ActionId::Reload => {
            state.status_message = "Reloading…".into();
            Some(ReducerEffect::ReGenerate)
        }
        ActionId::ToggleTooltip => {
            state.tooltip_visible = !state.tooltip_visible;
            Some(ReducerEffect::ReRender)
        }
        ActionId::StageLine | ActionId::StageHunk | ActionId::DiscardLine | ActionId::DiscardHunk => {
            dispatch_staging_action(state, action, ctx)
        }
        // Handled by dispatch_search_action in Search mode
        ActionId::SearchSubmit | ActionId::SearchCancel => None,
        ActionId::ToggleFocus => {
            if state.focus == FocusPane::Tree {
                state.focus = FocusPane::Diff;
            } else {
                state.tree_visible = true;
                state.focus = FocusPane::Tree;
                if state.tree_entries.is_empty() || state.tree_lines.is_empty() {
                    state.tree_entries = build_tree_entries(files);
                    let content_width = compute_tree_width(&state.tree_entries);
                    let has_directories = state.tree_entries.iter().any(|e| e.file_idx.is_none());
                    let file_count = state.doc.file_count();
                    let terminal_cols = ctx.cols as usize;
                    state.tree_width = resolve_tree_layout(content_width, terminal_cols, has_directories, file_count)
                        .unwrap_or_else(|| terminal_cols.saturating_sub(MIN_DIFF_WIDTH + 1));
                }
                let file_idx = state.doc.line_map.get(state.cursor_line).map_or(0, |li| li.file_idx);
                state.set_tree_cursor(file_idx_to_entry_idx(&state.tree_entries, file_idx));
                state.rebuild_tree_lines();
            }
            Some(ReducerEffect::ReRender)
        }
        ActionId::FocusDiff => {
            state.focus = FocusPane::Diff;
            Some(ReducerEffect::ReRender)
        }
        ActionId::FocusTree => {
            if state.focus == FocusPane::Tree && state.tree_visible {
                // Already focused on tree — close it
                state.tree_visible = false;
                state.focus = FocusPane::Diff;
            } else {
                // Open and focus tree
                state.tree_visible = true;
                state.focus = FocusPane::Tree;
                if state.tree_entries.is_empty() || state.tree_lines.is_empty() {
                    state.tree_entries = build_tree_entries(files);
                    let content_width = compute_tree_width(&state.tree_entries);
                    let has_directories = state.tree_entries.iter().any(|e| e.file_idx.is_none());
                    let file_count = state.doc.file_count();
                    let terminal_cols = ctx.cols as usize;
                    state.tree_width = resolve_tree_layout(content_width, terminal_cols, has_directories, file_count)
                        .unwrap_or_else(|| terminal_cols.saturating_sub(MIN_DIFF_WIDTH + 1));
                }
                let file_idx = state.doc.line_map.get(state.cursor_line).map_or(0, |li| li.file_idx);
                state.set_tree_cursor(file_idx_to_entry_idx(&state.tree_entries, file_idx));
                state.rebuild_tree_lines();
            }
            Some(ReducerEffect::ReRender)
        }
        ActionId::TreeEnter => {
            let sel = state.tree_selection?.get();
            let file_idx = state.tree_entries.get(sel)?.file_idx;
            match file_idx {
                None => {
                    state.tree_entries[sel].collapsed = !state.tree_entries[sel].collapsed;
                    state.rebuild_tree_lines();
                }
                Some(idx) => {
                    if state.active_file().is_some() {
                        set_view_to_file(state, idx, ch);
                    } else {
                        jump_to_tree_file(state, idx, ch);
                    }
                }
            }
            Some(ReducerEffect::Continue)
        }
    }
}

fn toggle_collapse_single(state: &mut PagerState) {
    let sel = match state.tree_selection {
        Some(s) => s.get(),
        None => return,
    };
    if state.tree_entries.get(sel).map_or(true, |e| e.file_idx.is_some()) {
        return;
    }
    let path = tree_entry_path(&state.tree_entries, sel);
    state.tree_entries[sel].collapsed = !state.tree_entries[sel].collapsed;
    if state.tree_entries[sel].collapsed {
        state.collapsed_paths.insert(path);
    } else {
        state.collapsed_paths.remove(&path);
    }
    state.rebuild_tree_lines();
}

fn toggle_collapse_recursive(state: &mut PagerState) {
    let sel = match state.tree_selection {
        Some(s) => s.get(),
        None => return,
    };
    let cursor_depth = match state.tree_entries.get(sel) {
        Some(e) if e.file_idx.is_none() => e.depth,
        _ => return,
    };
    let target = !state.tree_entries[sel].collapsed;

    // Apply to cursor entry
    state.tree_entries[sel].collapsed = target;
    let path = tree_entry_path(&state.tree_entries, sel);
    if target {
        state.collapsed_paths.insert(path);
    } else {
        state.collapsed_paths.remove(&path);
    }

    // Apply to all descendant directories
    for i in (sel + 1)..state.tree_entries.len() {
        if state.tree_entries[i].depth <= cursor_depth {
            break;
        }
        if state.tree_entries[i].file_idx.is_none() {
            state.tree_entries[i].collapsed = target;
            let p = tree_entry_path(&state.tree_entries, i);
            if target {
                state.collapsed_paths.insert(p);
            } else {
                state.collapsed_paths.remove(&p);
            }
        }
    }
    state.rebuild_tree_lines();
}

fn dispatch_staging_action(
    state: &mut PagerState,
    action: ActionId,
    ctx: &ReducerCtx<'_>,
) -> Option<ReducerEffect> {
    let files = ctx.files;

    // Guard: staging requires a diff line with file/hunk context
    let info = state.doc.line_map.get(state.cursor_line)?;
    if info.line_kind.is_none() {
        return Some(ReducerEffect::Continue);
    }
    let file_idx = info.file_idx;
    let hunk_idx = match info.hunk_idx {
        Some(h) => h,
        None => return Some(ReducerEffect::Continue),
    };

    // Guard: staging is not supported in commit or range views
    match ctx.source {
        DiffSource::Commit(_) | DiffSource::Range(_, _) => {
            state.status_message = "Cannot stage in this view".into();
            return Some(ReducerEffect::Continue);
        }
        _ => {}
    }

    // Determine cached/reverse flags based on action and source
    let is_stage = matches!(action, ActionId::StageLine | ActionId::StageHunk);
    let is_hunk = matches!(action, ActionId::StageHunk | ActionId::DiscardHunk);

    let (cached, reverse) = match ctx.source {
        DiffSource::WorkingTree => {
            if is_stage {
                (true, false)
            } else {
                (false, true)
            }
        }
        DiffSource::Staged => {
            if is_stage {
                // Unstage: apply cached reverse
                (true, true)
            } else {
                state.status_message = "Cannot discard in staged view".into();
                return Some(ReducerEffect::Continue);
            }
        }
        _ => unreachable!(),
    };

    let file = match files.get(file_idx) {
        Some(f) => f,
        None => return Some(ReducerEffect::Continue),
    };
    let hunk = match file.hunks.get(hunk_idx) {
        Some(h) => h,
        None => return Some(ReducerEffect::Continue),
    };

    let patch = if is_hunk {
        crate::git::patch::generate_hunk_patch(file, hunk)
    } else {
        // Line action: find which hunk line indices to select
        let selected = if let Some(anchor) = state.visual_anchor.take() {
            // Visual selection: collect all content-line indices in the range
            let lo = anchor.min(state.cursor_line);
            let hi = anchor.max(state.cursor_line);
            let mut indices = HashSet::new();
            for doc_line in lo..=hi {
                if let Some(li) = state.doc.line_map.get(doc_line) {
                    if li.file_idx == file_idx && li.hunk_idx == Some(hunk_idx) && li.line_kind.is_some() {
                        // Find the hunk line index by matching lineno
                        for (i, hl) in hunk.lines.iter().enumerate() {
                            if hl.old_lineno == li.old_lineno && hl.new_lineno == li.new_lineno {
                                indices.insert(i);
                                break;
                            }
                        }
                    }
                }
            }
            indices
        } else {
            // Single line: find the matching hunk line
            let mut indices = HashSet::new();
            for (i, hl) in hunk.lines.iter().enumerate() {
                if hl.old_lineno == info.old_lineno && hl.new_lineno == info.new_lineno {
                    indices.insert(i);
                    break;
                }
            }
            indices
        };

        if selected.is_empty() {
            return Some(ReducerEffect::Continue);
        }
        crate::git::patch::generate_line_patch(file, hunk, &selected)
    };

    Some(ReducerEffect::ApplyPatch { patch, cached, reverse })
}

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

fn reduce_search(
    state: &mut PagerState,
    event: &ReducerEvent,
    ctx: &ReducerCtx<'_>,
) -> ReducerEffect {
    let ReducerEvent::Key(key) = event;
    if let Some(action) = keymap_lookup(*key, KeyContext::Search) {
        return dispatch_search_action(state, action);
    }
    handle_search_key(state, *key);
    if state.mode == Mode::Normal && state.current_match >= 0 {
        scroll_to_match(state, ctx.content_height);
        sync_tree_cursor(state, ctx.content_height);
    }
    ReducerEffect::Continue
}

fn reduce_normal(
    state: &mut PagerState,
    event: &ReducerEvent,
    ctx: &ReducerCtx<'_>,
) -> ReducerEffect {
    let ReducerEvent::Key(key) = event;
    let ch = ctx.content_height;

    state.status_message.clear();

    if matches!(key, Key::Escape) && state.visual_anchor.is_some() {
        state.visual_anchor = None;
        return ReducerEffect::Continue;
    }

    if matches!(key, Key::Escape) && state.focus == FocusPane::Tree {
        state.focus = FocusPane::Diff;
        return ReducerEffect::Continue;
    }

    if state.focus == FocusPane::Tree {
        // Two-key sequence: z + a/A for collapse control
        if let Some('z') = state.pending_tree_key {
            state.pending_tree_key = None;
            match key {
                Key::Char('a') => {
                    toggle_collapse_single(state);
                    return ReducerEffect::Continue;
                }
                Key::Char('A') => {
                    toggle_collapse_recursive(state);
                    return ReducerEffect::Continue;
                }
                _ => {
                    // Cancel pending sequence
                    return ReducerEffect::Continue;
                }
            }
        }

        match key {
            Key::Char('z') => {
                state.pending_tree_key = Some('z');
                return ReducerEffect::Continue;
            }
            Key::Char('j') => {
                tree_cursor_down(state, ch);
                return ReducerEffect::Continue;
            }
            Key::Char('k') => {
                tree_cursor_up(state, ch);
                return ReducerEffect::Continue;
            }
            Key::Char('g') | Key::Home => {
                tree_cursor_top(state, ch);
                return ReducerEffect::Continue;
            }
            Key::Char('G') | Key::End => {
                tree_cursor_bottom(state, ch);
                return ReducerEffect::Continue;
            }
            Key::Enter | Key::Char(' ') => {
                if let Some(effect) = dispatch_normal_action(state, ActionId::TreeEnter, ctx) {
                    return effect;
                }
                return ReducerEffect::Continue;
            }
            _ => {}
        }
    }

    if let Some(action) = keymap_lookup(*key, KeyContext::Normal)
        && let Some(effect) = dispatch_normal_action(state, action, ctx)
    {
        return effect;
    }

    enforce_scrolloff(state, ch);
    sync_tree_cursor(state, ch);
    ReducerEffect::Continue
}

fn reduce(state: &mut PagerState, event: &ReducerEvent, ctx: &ReducerCtx<'_>) -> ReducerEffect {
    let effect = match state.mode {
        Mode::Search => reduce_search(state, event, ctx),
        Mode::Normal => reduce_normal(state, event, ctx),
    };
    clamp_cursor_and_top(state);
    debug_assert_valid_state(state);
    effect
}

pub(crate) fn handle_key(
    state: &mut PagerState,
    key: Key,
    ch: usize,
    rows: u16,
    cols: u16,
    files: &[DiffFile],
    repo: &std::path::Path,
    source: &crate::git::DiffSource,
) -> KeyResult {
    let event = ReducerEvent::Key(key);
    let ctx = ReducerCtx {
        content_height: ch,
        rows,
        cols,
        files,
        repo,
        source,
    };
    KeyResult::from(reduce(state, &event, &ctx))
}
