use tui::pager::{Key, copy_to_clipboard};

use crate::git::diff::DiffFile;

use super::content::{is_content_line, next_content_line, prev_content_line, snap_to_content};
use super::keymap::keymap_lookup;
use super::navigation::{
    nav_D_down, nav_U_up, nav_du_down, nav_du_up, recenter_top_line, sync_active_file_to_cursor,
    sync_tree_cursor, viewport_bounds,
};
use super::rendering::{enforce_scrolloff, format_copy_ref, resolve_lineno};
use super::search::{
    cancel_search, handle_search_key, next_match_in_range, prev_match_in_range, scroll_to_match,
    submit_search,
};
use super::state::{PagerState, ReducerCtx, ReducerEffect};
use super::state::{clamp_cursor_and_top, debug_assert_valid_state, visible_range};
use super::tree::{build_tree_entries, compute_tree_width, file_idx_to_entry_idx, resolve_tree_layout, MIN_DIFF_WIDTH};
use super::types::{ActionId, KeyContext, KeyResult, Mode};

#[derive(Debug, Clone)]
pub(crate) enum ReducerEvent {
    Key(Key),
}

fn set_view_to_file(state: &mut PagerState, file_idx: usize, ch: usize) {
    state.set_active_file(Some(file_idx));
    if let Some(start) = state.file_start(file_idx) {
        let file_end = state.file_end(file_idx).saturating_sub(1);
        state.top_line = start;
        state.cursor_line = snap_to_content(&state.doc.line_map, state.top_line, start, file_end);
        let (_, _, _, max_top) = viewport_bounds(state, ch);
        state.top_line = recenter_top_line(state.cursor_line, ch, start, max_top);
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
                    state.current_match =
                        (state.current_match + 1) % state.search_matches.len() as isize;
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
                    state.current_match = (state.current_match - 1
                        + state.search_matches.len() as isize)
                        % state.search_matches.len() as isize;
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
        ActionId::ToggleTooltip => {
            state.tooltip_visible = !state.tooltip_visible;
            Some(ReducerEffect::ReRender)
        }
        ActionId::SearchSubmit | ActionId::SearchCancel => None,
    }
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
) -> KeyResult {
    let event = ReducerEvent::Key(key);
    let ctx = ReducerCtx {
        content_height: ch,
        rows,
        cols,
        files,
        repo,
    };
    KeyResult::from(reduce(state, &event, &ctx))
}
