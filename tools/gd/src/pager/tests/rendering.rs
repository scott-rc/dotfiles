//! Characterization tests for `pager::rendering` helper functions.

use super::super::rendering::*;
use super::super::types::Mode;
use super::common::{
    make_keybinding_state, make_line_map, make_pager_state_for_range, make_pager_state_from_files,
    make_two_file_diff, strip,
};
use crate::git::diff::LineKind;
use crate::render::LineInfo;
use crate::style;

#[test]
fn diff_area_width_tree_hidden_no_scrollbar() {
    assert_eq!(diff_area_width(80, 10, false, false), 80);
}

#[test]
fn diff_area_width_tree_visible_no_scrollbar() {
    assert_eq!(diff_area_width(80, 10, true, false), 69);
}

#[test]
fn diff_area_width_tree_visible_scrollbar() {
    assert_eq!(diff_area_width(80, 10, true, true), 68);
}

#[test]
fn diff_area_width_narrow_terminal_saturates_to_zero() {
    assert_eq!(diff_area_width(5, 10, true, true), 0);
}

// -- bar_visible --

#[test]
fn bar_visible_normal_mode_is_false() {
    let state = make_keybinding_state();
    assert!(!bar_visible(&state), "Normal mode should hide bar");
}

#[test]
fn bar_visible_search_mode_is_true() {
    let mut state = make_keybinding_state();
    state.mode = Mode::Search;
    assert!(bar_visible(&state), "bar should be visible in Search");
}

#[test]
fn bar_visible_status_message_any_mode() {
    let mut state = make_keybinding_state();
    state.status_message = "some message".into();
    assert!(
        bar_visible(&state),
        "non-empty status_message should show bar in Normal mode"
    );
}

#[test]
fn bar_visible_tooltip_visible() {
    let mut state = make_keybinding_state();
    state.tooltip_visible = true;
    assert!(bar_visible(&state), "tooltip_visible should show bar");
}

// -- content_height --

#[test]
fn content_height_normal_mode_full_rows() {
    let state = make_keybinding_state();
    assert_eq!(content_height(24, &state), 24);
}

#[test]
fn content_height_bar_visible_subtracts_one() {
    let mut state = make_keybinding_state();
    state.mode = Mode::Search;
    assert_eq!(content_height(24, &state), 23);
}

#[test]
fn content_height_tooltip_visible_subtracts_tooltip_height() {
    let mut state = make_keybinding_state();
    state.tooltip_visible = true;
    assert_eq!(content_height(24, &state), 24 - 1 - TOOLTIP_HEIGHT);
}

// -- resolve_lineno --

fn li(new: Option<u32>, old: Option<u32>) -> LineInfo {
    LineInfo {
        file_idx: 0,
        path: "test.rs".into(),
        new_lineno: new,
        old_lineno: old,
        line_kind: Some(LineKind::Context),
    }
}

#[test]
fn resolve_lineno_prefers_new_lineno() {
    let map = vec![
        li(Some(10), Some(100)),
        li(Some(11), Some(101)),
        li(Some(12), Some(102)),
    ];
    assert_eq!(resolve_lineno(&map, 0, 2), (Some(10), Some(12)));
}

#[test]
fn resolve_lineno_falls_back_to_old_lineno() {
    let map = vec![li(None, Some(50)), li(None, Some(51)), li(None, Some(52))];
    assert_eq!(resolve_lineno(&map, 0, 2), (Some(50), Some(52)));
}

#[test]
fn resolve_lineno_no_linenos_returns_none() {
    let map = vec![li(None, None), li(None, None)];
    assert_eq!(resolve_lineno(&map, 0, 1), (None, None));
}

#[test]
fn resolve_lineno_single_line_range() {
    let map = vec![li(None, None), li(Some(42), Some(99)), li(None, None)];
    assert_eq!(resolve_lineno(&map, 1, 1), (Some(42), Some(42)));
}

// -- format_copy_ref --

#[test]
fn format_copy_ref_same_start_end() {
    assert_eq!(format_copy_ref("path", Some(5), Some(5)), "path:5");
}

#[test]
fn format_copy_ref_different_start_end() {
    assert_eq!(format_copy_ref("path", Some(5), Some(10)), "path:5-10");
}

#[test]
fn format_copy_ref_start_only() {
    assert_eq!(format_copy_ref("path", Some(5), None), "path:5");
}

#[test]
fn format_copy_ref_no_linenos() {
    assert_eq!(format_copy_ref("path", None, None), "path");
}

// -- enforce_scrolloff --

#[test]
fn enforce_scrolloff_cursor_near_top() {
    let mut state = make_pager_state_for_range(vec![0], 50, None);
    state.cursor_line = 2;
    state.top_line = 0;
    enforce_scrolloff(&mut state, 20);
    assert_eq!(
        state.top_line, 0,
        "top should stay at 0 since cursor-SCROLLOFF saturates to 0"
    );
}

#[test]
fn enforce_scrolloff_cursor_near_bottom() {
    let mut state = make_pager_state_for_range(vec![0], 50, None);
    state.cursor_line = 45;
    state.top_line = 20;
    enforce_scrolloff(&mut state, 20);
    assert!(
        state.top_line >= 26,
        "top_line should scroll down so cursor+SCROLLOFF fits: got {}",
        state.top_line
    );
}

#[test]
fn enforce_scrolloff_cursor_in_middle_no_change() {
    let mut state = make_pager_state_for_range(vec![0], 50, None);
    state.cursor_line = 20;
    state.top_line = 10;
    let original_top = state.top_line;
    enforce_scrolloff(&mut state, 20);
    assert_eq!(
        state.top_line, original_top,
        "cursor well within scrolloff — no adjustment"
    );
}

#[test]
fn enforce_scrolloff_cursor_at_range_boundaries() {
    let mut state = make_pager_state_for_range(vec![0], 50, None);
    state.cursor_line = 49;
    state.top_line = 0;
    enforce_scrolloff(&mut state, 20);
    assert_eq!(
        state.cursor_line, 49,
        "cursor should be clamped to last line"
    );
    assert!(
        state.top_line <= 30,
        "top_line should be within max_top: got {}",
        state.top_line
    );
}

// -- render_scrollbar_cell --

#[test]
fn render_scrollbar_cell_thumb_vs_track() {
    let map = make_line_map(&[Some(LineKind::Context); 100]);
    let ch = 20;
    let (vis_start, vis_end) = (0, 100);
    let top = 0;
    let cell_in_thumb = render_scrollbar_cell(0, ch, vis_start, vis_end, top, &map);
    assert!(
        cell_in_thumb.contains(style::BG_SCROLLBAR_THUMB),
        "row 0 at top=0 should be in thumb"
    );
    let cell_outside = render_scrollbar_cell(ch - 1, ch, vis_start, vis_end, top, &map);
    assert!(
        cell_outside.contains(style::BG_SCROLLBAR_TRACK),
        "last row at top=0 should be track"
    );
}

#[test]
fn render_scrollbar_cell_added_and_deleted_indicators() {
    let mut kinds: Vec<Option<LineKind>> = vec![Some(LineKind::Context); 20];
    kinds[0] = Some(LineKind::Added);
    kinds[10] = Some(LineKind::Deleted);
    let map = make_line_map(&kinds);
    let ch = 20;

    let added_cell = render_scrollbar_cell(0, ch, 0, 20, 0, &map);
    assert!(
        added_cell.contains('\u{2590}') && added_cell.contains(style::FG_ADDED_MARKER),
        "added line should show added indicator: {added_cell:?}"
    );

    let deleted_cell = render_scrollbar_cell(10, ch, 0, 20, 0, &map);
    assert!(
        deleted_cell.contains('\u{2590}') && deleted_cell.contains(style::FG_DELETED_MARKER),
        "deleted line should show deleted indicator: {deleted_cell:?}"
    );
}

#[test]
fn render_scrollbar_cell_zero_range_returns_track() {
    let map = make_line_map(&[Some(LineKind::Context)]);
    let cell = render_scrollbar_cell(0, 20, 5, 5, 5, &map);
    assert!(
        cell.contains(style::BG_SCROLLBAR_TRACK),
        "zero range should return track: {cell:?}"
    );
    assert!(
        !cell.contains(style::BG_SCROLLBAR_THUMB),
        "zero range should not contain thumb: {cell:?}"
    );
}

// -- format_status_bar --

#[test]
fn format_status_bar_status_message() {
    let mut state = make_keybinding_state();
    state.status_message = "Copied to clipboard".into();
    let bar = format_status_bar(&state, 20, 80);
    let visible = strip(&bar);
    assert!(
        visible.contains("Copied to clipboard"),
        "should contain status message: {visible:?}"
    );
    assert_eq!(visible.len(), 80, "should be padded to cols");
}

#[test]
fn format_status_bar_position_indicator() {
    let state = make_keybinding_state();
    let bar = format_status_bar(&state, 20, 80);
    let visible = strip(&bar);
    assert!(
        visible.contains("TOP") || visible.contains("END") || visible.contains('%'),
        "should contain position indicator: {visible:?}"
    );
}

#[test]
fn format_status_bar_visual_mode() {
    let mut state = make_keybinding_state();
    state.visual_anchor = Some(5);
    let bar = format_status_bar(&state, 20, 80);
    let visible = strip(&bar);
    assert!(
        visible.contains("VISUAL"),
        "should show visual mode indicator: {visible:?}"
    );
}

#[test]
fn format_status_bar_single_file() {
    let mut state = make_keybinding_state();
    state.set_active_file(Some(0));
    let bar = format_status_bar(&state, 20, 80);
    let visible = strip(&bar);
    assert!(
        visible.contains("Single:"),
        "should show single file indicator: {visible:?}"
    );
}

// -- format_tooltip_lines --

#[test]
fn format_tooltip_lines_produces_two_lines() {
    let lines = format_tooltip_lines(80);
    assert_eq!(
        lines.len(),
        TOOLTIP_HEIGHT,
        "tooltip should produce exactly TOOLTIP_HEIGHT lines"
    );
}

#[test]
fn format_tooltip_lines_contains_key_hints() {
    let lines = format_tooltip_lines(80);
    let joined = lines.join(" ");
    let stripped = strip(&joined);
    assert!(stripped.contains("j/k"), "tooltip should mention j/k");
    assert!(stripped.contains("quit"), "tooltip should mention quit");
}

#[test]
fn format_tooltip_lines_narrow_width_is_visible_width_safe() {
    let cols = 20usize;
    let lines = format_tooltip_lines(cols);
    for line in lines {
        let visible = strip(&line);
        assert!(
            visible.chars().count() <= cols,
            "tooltip line visible width must be <= cols: {visible:?}"
        );
    }
}

// -- render_content_area --

#[test]
fn render_content_area_basic() {
    let files = make_two_file_diff();
    let state = make_pager_state_from_files(&files, false);
    let mut buf = Vec::new();
    render_content_area(&mut buf, &state, 80, 24);
    assert!(!buf.is_empty(), "should produce output");
    let output = String::from_utf8_lossy(&buf);
    assert!(
        output.contains("\x1b["),
        "should contain ANSI escape sequences: {output:?}"
    );
}

#[test]
fn render_content_area_search_highlight() {
    let files = make_two_file_diff();
    let mut state = make_pager_state_from_files(&files, false);
    state.search_query = "first".into();
    let mut buf = Vec::new();
    render_content_area(&mut buf, &state, 80, 24);
    let output = String::from_utf8_lossy(&buf);
    assert!(
        output.contains("\x1b["),
        "search should produce ANSI highlighting: {output:?}"
    );
}

#[test]
fn render_content_area_visual_highlight() {
    let files = make_two_file_diff();
    let mut state = make_pager_state_from_files(&files, false);
    state.visual_anchor = Some(0);
    state.cursor_line = 2;
    let mut buf = Vec::new();
    render_content_area(&mut buf, &state, 80, 24);
    let output = String::from_utf8_lossy(&buf);
    assert!(
        output.contains(style::BG_VISUAL),
        "visual selection should render highlight: {output:?}"
    );
}

// -- render_screen --

#[test]
fn render_screen_no_status_bar() {
    let state = make_keybinding_state();
    let mut buf = Vec::new();
    render_screen(&mut buf, &state, 80, 24);
    assert!(
        !buf.is_empty(),
        "render_screen in Normal mode should produce output"
    );
}

#[test]
fn render_screen_with_tooltip() {
    let mut state = make_keybinding_state();
    state.tooltip_visible = true;
    let mut buf = Vec::new();
    render_screen(&mut buf, &state, 80, 24);
    let output = String::from_utf8_lossy(&buf);
    let visible = strip(&output);
    assert!(
        visible.contains("quit"),
        "tooltip render_screen should include key hints: {visible:?}"
    );
}
