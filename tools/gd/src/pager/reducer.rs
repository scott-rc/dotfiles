use tui::pager::Key;

use crate::git::diff::DiffFile;

use super::content::{is_content_line, next_content_line, prev_content_line, snap_to_content};
use super::keymap::keymap_lookup;
use super::navigation::{
    ensure_tree_cursor_visible,
    move_tree_selection,
    nav_D_down,
    nav_U_up,
    nav_du_down,
    nav_du_up,
    recenter_top_line,
    sync_active_file_to_cursor,
    sync_tree_cursor,
    sync_tree_cursor_force,
    viewport_bounds,
};
use super::rendering::enforce_scrolloff;
use super::search::{handle_search_key, scroll_to_match, submit_search, cancel_search};
use super::state::{debug_assert_valid_state, clamp_cursor_and_top, visible_range};
use super::state::{PagerState, ReducerCtx, ReducerEffect};
use super::tree::{build_tree_entries, build_tree_lines, compute_tree_width, file_idx_to_entry_idx};
use super::types::{ActionId, KeyContext, KeyResult, Mode};

#[derive(Debug, Clone)]
pub(crate) enum ReducerEvent {
    Key(Key),
}

fn dispatch_normal_action(
    state: &mut PagerState,
    action: ActionId,
    ctx: &ReducerCtx<'_>,
) -> Option<ReducerEffect> {
    use ActionId::*;
    let ch = ctx.content_height;
    let _rows = ctx.rows;
    let files = ctx.files;
    let half_page = ch / 2;
    let (range_start, range_end) = visible_range(state);
    let max_cursor = range_end.saturating_sub(1);

    match action {
        Quit => return Some(ReducerEffect::Quit),
        ScrollDown => {
            let next = (state.cursor_line + 1).min(max_cursor);
            state.cursor_line = next_content_line(&state.doc.line_map, next, max_cursor);
            return None;
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
            state.status_message = res.status_message.clone();
            sync_tree_cursor(state, ch);
            return Some(ReducerEffect::Continue);
        }
        PrevFile => {
            let res = nav_U_up(state, ch);
            state.cursor_line = res.cursor_line;
            state.top_line = res.top_line;
            state.status_message = res.status_message.clone();
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
                        scroll_to_match(state, ch);
                    }
                } else {
                    state.current_match =
                        (state.current_match + 1) % state.search_matches.len() as isize;
                    scroll_to_match(state, ch);
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
                        scroll_to_match(state, ch);
                    }
                } else {
                    state.current_match = (state.current_match - 1
                        + state.search_matches.len() as isize)
                        % state.search_matches.len() as isize;
                    scroll_to_match(state, ch);
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
                let (tl, tv) = build_tree_lines(
                    &state.tree_entries,
                    state.tree_cursor(),
                    state.tree_width,
                );
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
    use tui::pager::copy_to_clipboard;

    let ch = ctx.content_height;

    match action {
        VisualExtendDown => {
            let next = state.cursor_line + 1;
            let anchor_file = state
                .doc
                .line_map
                .get(state.visual_anchor)
                .map_or(0, |l| l.file_idx);
            let next_file = state
                .doc
                .line_map
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
                    .doc
                    .line_map
                    .get(state.visual_anchor)
                    .map_or(0, |l| l.file_idx);
                let prev_file = state
                    .doc
                    .line_map
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
                .doc
                .line_map
                .get(lo)
                .map(|l| l.path.clone())
                .unwrap_or_default();
            let (start, end) = super::rendering::resolve_lineno(&state.doc.line_map, lo, hi);
            let text = super::rendering::format_copy_ref(&path, start, end);
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
    handle_search_key(state, key);
    if state.mode == Mode::Normal && state.current_match >= 0 {
        scroll_to_match(state, ctx.content_height);
        sync_tree_cursor(state, ctx.content_height);
    }
    ReducerEffect::Continue
}

fn reduce_help(
    state: &mut PagerState,
    _event: &ReducerEvent,
    _ctx: &ReducerCtx<'_>,
) -> ReducerEffect {
    state.mode = Mode::Normal;
    ReducerEffect::Continue
}

fn reduce_visual(
    state: &mut PagerState,
    event: &ReducerEvent,
    ctx: &ReducerCtx<'_>,
) -> ReducerEffect {
    let ReducerEvent::Key(key) = event;
    if let Some(action) = keymap_lookup(*key, KeyContext::Visual) {
        return dispatch_visual_action(state, action, ctx);
    }
    ReducerEffect::Continue
}

fn reduce_tree(
    state: &mut PagerState,
    event: &ReducerEvent,
    ctx: &ReducerCtx<'_>,
) -> ReducerEffect {
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

fn reduce(
    state: &mut PagerState,
    event: ReducerEvent,
    ctx: &ReducerCtx<'_>,
) -> ReducerEffect {
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
