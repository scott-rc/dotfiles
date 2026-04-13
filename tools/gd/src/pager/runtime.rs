use std::io::{self, Write};
use std::time::{Duration, Instant, SystemTime};

use crossterm::event::{
    self, Event, KeyEventKind, KeyboardEnhancementFlags, PopKeyboardEnhancementFlags,
    PushKeyboardEnhancementFlags,
};

use tui::pager::{
    ALT_SCREEN_OFF, ALT_SCREEN_ON, CURSOR_HIDE, CURSOR_SHOW, Key, crossterm_to_key, get_term_size,
};

use crate::git::diff::DiffFile;

use super::reducer::handle_key;
use super::rendering::{content_height, render_screen};
use super::state::{DiffContext, ReducerCtx};
use super::state::{Document, PagerState, capture_view_anchor, remap_after_document_swap};
use super::tree::{build_tree_entries, compute_tree_width, resolve_tree_layout};
use super::types::KeyResult;

fn gd_debug_enabled() -> bool {
    std::env::var_os("GD_DEBUG").is_some_and(|v| v == "1")
}

fn debug_escape(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

fn debug_trace(location: &str, message: &str, data: &str) {
    if !gd_debug_enabled() {
        return;
    }
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_millis() as u64);
    let line = format!(
        "[gd] {{\"location\":\"{}\",\"message\":\"{}\",\"data\":{},\"timestamp\":{}}}\n",
        debug_escape(location),
        debug_escape(message),
        data,
        ts
    );
    let _ = std::io::stderr().write_all(line.as_bytes());
}

/// Terminal columns adjusted for scrollbar (full_context reserves 1 column).
fn effective_terminal_cols(cols: usize, full_context: bool) -> usize {
    if full_context {
        cols.saturating_sub(1)
    } else {
        cols
    }
}

/// Compute the diff area width for the initial render, accounting for tree
/// panel and scrollbar. Avoids a redundant Phase 2 relayout after startup.
fn initial_render_width(
    cols: u16,
    tree_entries: &[super::tree::TreeEntry],
    file_count: usize,
    full_context: bool,
) -> usize {
    use super::tree::{compute_tree_width, resolve_tree_layout};

    let effective = effective_terminal_cols(cols as usize, full_context);
    let content_width = compute_tree_width(tree_entries);
    let has_dirs = tree_entries.iter().any(|e| e.file_idx.is_none());
    let tree_layout = resolve_tree_layout(content_width, effective, has_dirs, file_count);
    let tree_visible = tree_layout.is_some();
    let tree_width = tree_layout.unwrap_or(0);
    super::rendering::diff_area_width(cols, tree_width, tree_visible, full_context)
}

/// Emit a per-keystroke timing trace (only when `GD_DEBUG=1`).
fn trace_keystroke(key: Key, key_dur: Duration, render_dur: Duration) {
    let total = key_dur + render_dur;
    debug_trace(
        "runtime:keystroke",
        "per-keystroke timing",
        &format!(
            "{{\"key\":\"{:?}\",\"handleKeyMs\":{:.2},\"renderScreenMs\":{:.2},\"totalMs\":{:.2}}}",
            key,
            key_dur.as_secs_f64() * 1000.0,
            render_dur.as_secs_f64() * 1000.0,
            total.as_secs_f64() * 1000.0,
        ),
    );
}

fn format_debug_state(state: &PagerState) -> String {
    let (rs, re) = super::state::visible_range(state);
    let active_file_valid = state
        .active_file()
        .is_none_or(|idx| idx < state.doc.file_starts.len());
    let tree_cursor_file_idx = state
        .tree_entries
        .get(state.tree_cursor())
        .and_then(|e| e.file_idx)
        .map_or(String::from("null"), |v| v.to_string());
    format!(
        "{{\"treeVisible\":{},\"activeFile\":{},\"activeFileValid\":{},\"fullContext\":{},\"cursorLine\":{},\"topLine\":{},\"rangeStart\":{},\"rangeEnd\":{},\"lineMapLen\":{},\"fileStartsLen\":{},\"treeCursor\":{},\"treeCursorFileIdx\":{}}}",
        state.tree_visible,
        state
            .active_file()
            .map_or(String::from("null"), |v| v.to_string()),
        active_file_valid,
        state.full_context,
        state.cursor_line,
        state.top_line,
        rs,
        re,
        state.doc.line_map.len(),
        state.doc.file_starts.len(),
        state.tree_cursor(),
        tree_cursor_file_idx
    )
}

fn open_in_editor(path: &str, line: Option<u32>) {
    tui::pager::open_in_editor(path, line.map(|l| l as usize), false);
}

pub(crate) fn git_index_mtime(repo: &std::path::Path) -> Option<SystemTime> {
    std::fs::metadata(repo.join(".git/index"))
        .ok()
        .and_then(|m| m.modified().ok())
}

pub(crate) fn resolve_path_for_editor(path: &str, repo: &std::path::Path) -> std::path::PathBuf {
    let file = std::path::Path::new(path);
    if file.is_absolute() {
        file.to_path_buf()
    } else {
        repo.join(file)
    }
}

#[derive(Clone, Copy)]
enum RenderMode {
    Relayout,
    Full,
}

fn sync_tree_layout(state: &mut PagerState, files: &[DiffFile], cols: u16) {
    if files.is_empty() {
        return;
    }

    let entries = build_tree_entries(files);
    let content_width = compute_tree_width(&entries);
    let has_directories = entries.iter().any(|e| e.file_idx.is_none());
    let file_count = state.doc.file_count().max(files.len());
    let effective_cols = effective_terminal_cols(cols as usize, state.full_context);

    if let Some(width) =
        resolve_tree_layout(content_width, effective_cols, has_directories, file_count)
    {
        if state.tree_visible {
            state.tree_width = width;
            state.tree_entries = entries;
        } else if !state.tree_user_hidden {
            state.tree_visible = true;
            state.tree_width = width;
            state.tree_entries = entries;
        }
    } else if state.tree_visible {
        state.tree_visible = false;
        state.tree_width = 0;
    }
}

fn rerender_document(
    state: &mut PagerState,
    files: &[DiffFile],
    color: bool,
    cols: u16,
    mode: RenderMode,
    trace_location: &str,
) {
    let anchor = capture_view_anchor(state);
    sync_tree_layout(state, files, cols);

    let width = super::rendering::diff_area_width(
        cols,
        state.tree_width,
        state.tree_visible,
        state.full_context,
    );
    let output = match mode {
        RenderMode::Relayout => {
            let styled_files = std::mem::take(&mut state.doc.styled_files);
            crate::render::relayout(styled_files, width, color)
        }
        RenderMode::Full => crate::render::render(files, width, color),
    };
    let new_doc = Document::from_render_output(output);
    remap_after_document_swap(state, anchor.as_ref(), new_doc, files, cols as usize);

    debug_trace(
        trace_location,
        "post render state",
        &format_debug_state(state),
    );
}

pub(crate) fn re_render(state: &mut PagerState, files: &[DiffFile], color: bool, cols: u16) {
    rerender_document(
        state,
        files,
        color,
        cols,
        RenderMode::Relayout,
        "runtime:re_render",
    );
}

/// Full render: re-style and relayout (for content changes).
pub(crate) fn full_render(state: &mut PagerState, files: &[DiffFile], color: bool, cols: u16) {
    rerender_document(
        state,
        files,
        color,
        cols,
        RenderMode::Full,
        "runtime:full_render",
    );
}

fn regenerate_files(diff_ctx: &DiffContext, full_context: bool) -> Vec<DiffFile> {
    let mut diff_args = if full_context {
        diff_ctx.source.diff_args_full_context()
    } else {
        diff_ctx.source.diff_args()
    };
    if diff_ctx.ignore_whitespace {
        diff_args.push("-w".into());
    }
    let str_args: Vec<&str> = diff_args.iter().map(String::as_str).collect();
    let raw = crate::git::run_diff(&diff_ctx.repo, &str_args);
    let mut files = crate::git::diff::parse(&raw);

    crate::git::append_untracked(
        &diff_ctx.repo,
        &diff_ctx.source,
        diff_ctx.no_untracked,
        &mut files,
    );
    crate::git::sort_files_for_display(&mut files);

    files
}

fn initialize_pager_state(
    files: &[DiffFile],
    color: bool,
    use_full_context: bool,
    cols: u16,
    rows: u16,
) -> PagerState {
    let tree_entries = build_tree_entries(files);
    let effective_cols = effective_terminal_cols(cols as usize, use_full_context);
    let render_width = initial_render_width(cols, &tree_entries, files.len(), use_full_context);

    let output = crate::render::render(files, render_width, color);
    let doc = Document::from_render_output(output);
    let mut state = PagerState::from_doc(doc, tree_entries, effective_cols);
    state.full_context = use_full_context;
    state.view_scope =
        super::state::default_view_scope(files.len(), state.doc.line_count(), rows as usize);
    state
}

fn reducer_ctx<'a>(
    state: &PagerState,
    files: &'a [DiffFile],
    diff_ctx: &'a DiffContext,
    cols: u16,
    rows: u16,
) -> ReducerCtx<'a> {
    ReducerCtx {
        content_height: content_height(rows, state),
        rows,
        cols,
        files,
        repo: &diff_ctx.repo,
        source: &diff_ctx.source,
    }
}

fn refresh_files_and_render(
    files: &mut Vec<DiffFile>,
    state: &mut PagerState,
    diff_ctx: &DiffContext,
    color: bool,
    cols: u16,
) -> bool {
    *files = regenerate_files(diff_ctx, state.full_context);
    if files.is_empty() {
        return false;
    }
    full_render(state, files, color, cols);
    true
}

fn refresh_files_render_and_index(
    files: &mut Vec<DiffFile>,
    state: &mut PagerState,
    diff_ctx: &DiffContext,
    color: bool,
    cols: u16,
    last_index_mtime: &mut Option<SystemTime>,
) -> bool {
    if !refresh_files_and_render(files, state, diff_ctx, color, cols) {
        return false;
    }
    *last_index_mtime = git_index_mtime(&diff_ctx.repo);
    true
}

fn trace_regenerate_state(location: &str, message: &str, state: &PagerState, files: &[DiffFile]) {
    let base = format_debug_state(state);
    debug_trace(
        location,
        message,
        &format!(
            "{},\"filesLen\":{}}}",
            base.trim_end_matches('}'),
            files.len()
        ),
    );
}

fn apply_patch_result(
    repo: &std::path::Path,
    patch: &str,
    cached: bool,
    reverse: bool,
) -> Result<(), String> {
    if cached && !reverse {
        crate::git::stage_patch(repo, patch)
    } else if cached && reverse {
        crate::git::unstage_patch(repo, patch)
    } else {
        crate::git::revert_patch(repo, patch)
    }
}

pub(crate) fn parse_replay_keys(input: &str) -> Vec<Key> {
    let mut keys = Vec::new();
    let mut chars = input.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '<' {
            let mut name = String::new();
            for nc in chars.by_ref() {
                if nc == '>' {
                    break;
                }
                name.push(nc);
            }
            keys.push(match name.as_str() {
                "Enter" | "CR" => Key::Enter,
                "Esc" => Key::Escape,
                "Tab" => Key::Tab,
                "BS" | "Backspace" => Key::Backspace,
                "Up" => Key::Up,
                "Down" => Key::Down,
                "Left" => Key::Left,
                "Right" => Key::Right,
                "PgUp" | "PageUp" => Key::PageUp,
                "PgDn" | "PageDown" => Key::PageDown,
                "Home" => Key::Home,
                "End" => Key::End,
                "C-c" => Key::CtrlC,
                "C-d" => Key::CtrlD,
                "C-e" => Key::CtrlE,
                "C-h" => Key::CtrlH,
                "C-l" => Key::CtrlL,
                "C-u" => Key::CtrlU,
                _ => Key::Unknown,
            });
        } else if c == '\\' {
            match chars.peek() {
                Some('n') => {
                    chars.next();
                    keys.push(Key::Enter);
                }
                Some('t') => {
                    chars.next();
                    keys.push(Key::Tab);
                }
                Some('\\') => {
                    chars.next();
                    keys.push(Key::Char('\\'));
                }
                Some('<') => {
                    chars.next();
                    keys.push(Key::Char('<'));
                }
                _ => keys.push(Key::Char('\\')),
            }
        } else {
            keys.push(Key::Char(c));
        }
    }
    keys
}

pub fn run_pager(
    files: Vec<DiffFile>,
    color: bool,
    use_full_context: bool,
    diff_ctx: &DiffContext,
) {
    let t0 = std::time::Instant::now();
    let mut files = files;
    let mut stdout = io::BufWriter::new(io::stdout());

    let _ = write!(stdout, "{ALT_SCREEN_ON}{CURSOR_HIDE}");
    let _ = stdout.flush();
    let _ = crossterm::terminal::enable_raw_mode();
    let _ = crossterm::execute!(
        stdout,
        PushKeyboardEnhancementFlags(KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES)
    );

    let prev_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let mut out = io::stdout();
        let _ = crossterm::execute!(out, PopKeyboardEnhancementFlags);
        let _ = crossterm::terminal::disable_raw_mode();
        let _ = out.write_all(format!("{CURSOR_SHOW}{ALT_SCREEN_OFF}").as_bytes());
        let _ = out.flush();
        prev_hook(info);
    }));

    let mut last_size = get_term_size();
    let mut state =
        initialize_pager_state(&files, color, use_full_context, last_size.0, last_size.1);
    crate::debug::trace("pager", "post-render", t0);

    render_screen(&mut stdout, &state, last_size.0, last_size.1);
    debug_trace(
        "runtime:render_screen",
        "post initial render",
        &format!("{{\"totalLines\":{}}}", state.doc.line_count()),
    );

    let mut last_index_mtime = git_index_mtime(&diff_ctx.repo);
    let mut last_poll_check = Instant::now();

    loop {
        let ev = match event::poll(Duration::from_millis(50)) {
            Ok(true) => match event::read() {
                Ok(ev) => ev,
                Err(_) => break,
            },
            Ok(false) => {
                let current_size = get_term_size();
                let mut needs_render = false;
                if current_size != last_size {
                    last_size = current_size;
                    re_render(&mut state, &files, color, last_size.0);
                    needs_render = true;
                }
                if last_poll_check.elapsed() >= Duration::from_secs(2) {
                    last_poll_check = Instant::now();
                    let current_mtime = git_index_mtime(&diff_ctx.repo);
                    if current_mtime != last_index_mtime {
                        last_index_mtime = current_mtime;
                        if !refresh_files_and_render(
                            &mut files,
                            &mut state,
                            diff_ctx,
                            color,
                            last_size.0,
                        ) {
                            break;
                        }
                        needs_render = true;
                    }
                }
                if needs_render {
                    render_screen(&mut stdout, &state, last_size.0, last_size.1);
                }
                continue;
            }
            Err(_) => break,
        };

        let key = match ev {
            Event::Resize(_, _) => {
                last_size = get_term_size();
                re_render(&mut state, &files, color, last_size.0);
                render_screen(&mut stdout, &state, last_size.0, last_size.1);
                continue;
            }
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                crossterm_to_key(key_event)
            }
            _ => continue,
        };

        let ctx = reducer_ctx(&state, &files, diff_ctx, last_size.0, last_size.1);
        let debug = gd_debug_enabled();
        let t_key_start = Instant::now();
        let result = handle_key(&mut state, key, &ctx);
        let key_dur = t_key_start.elapsed();

        match result {
            KeyResult::Quit => break,
            KeyResult::ReRender => {
                re_render(&mut state, &files, color, last_size.0);
            }
            KeyResult::ReGenerate => {
                trace_regenerate_state(
                    "runtime:run_pager:regenerate:before",
                    "regenerate start",
                    &state,
                    &files,
                );
                if !refresh_files_render_and_index(
                    &mut files,
                    &mut state,
                    diff_ctx,
                    color,
                    last_size.0,
                    &mut last_index_mtime,
                ) {
                    break;
                }
                trace_regenerate_state(
                    "runtime:run_pager:regenerate:after",
                    "regenerate complete",
                    &state,
                    &files,
                );
            }
            KeyResult::OpenEditor { path, lineno } => {
                let _ = crossterm::terminal::disable_raw_mode();
                let _ = write!(stdout, "{CURSOR_SHOW}{ALT_SCREEN_OFF}");
                let _ = stdout.flush();

                let resolved = resolve_path_for_editor(&path, &diff_ctx.repo);
                open_in_editor(resolved.to_string_lossy().as_ref(), lineno);

                let _ = write!(stdout, "{ALT_SCREEN_ON}{CURSOR_HIDE}");
                let _ = stdout.flush();
                let _ = crossterm::terminal::enable_raw_mode();
                last_size = get_term_size();

                if !refresh_files_render_and_index(
                    &mut files,
                    &mut state,
                    diff_ctx,
                    color,
                    last_size.0,
                    &mut last_index_mtime,
                ) {
                    break;
                }
            }
            KeyResult::ApplyPatch {
                patch,
                cached,
                reverse,
            } => match apply_patch_result(&diff_ctx.repo, &patch, cached, reverse) {
                Ok(()) => {
                    if !refresh_files_render_and_index(
                        &mut files,
                        &mut state,
                        diff_ctx,
                        color,
                        last_size.0,
                        &mut last_index_mtime,
                    ) {
                        break;
                    }
                }
                Err(e) => {
                    state.status_message =
                        format!("Apply failed: {}", e.lines().next().unwrap_or(&e));
                }
            },
            KeyResult::Continue => {}
        }

        let t_render_start = Instant::now();
        render_screen(&mut stdout, &state, last_size.0, last_size.1);
        let render_dur = t_render_start.elapsed();

        if debug {
            trace_keystroke(key, key_dur, render_dur);
        }
    }

    let _ = crossterm::execute!(stdout, PopKeyboardEnhancementFlags);
    let _ = crossterm::terminal::disable_raw_mode();
    let _ = write!(stdout, "{CURSOR_SHOW}{ALT_SCREEN_OFF}");
    let _ = stdout.flush();

    // Remove the custom panic hook (restores the default hook).
    let _ = std::panic::take_hook();
}

/// Replay a sequence of keystrokes through the full pager pipeline without a TTY.
/// Renders to an in-memory buffer; combined with `GD_DEBUG=1`, emits per-keystroke
/// timing traces to stderr.
pub fn run_pager_replay(
    files: Vec<DiffFile>,
    color: bool,
    use_full_context: bool,
    diff_ctx: &DiffContext,
    keys_str: &str,
    cols: u16,
    rows: u16,
) {
    let t0 = std::time::Instant::now();
    let mut files = files;
    let mut sink = Vec::<u8>::with_capacity(64 * 1024);
    let mut state = initialize_pager_state(&files, color, use_full_context, cols, rows);
    crate::debug::trace("pager", "post-render", t0);

    render_screen(&mut sink, &state, cols, rows);
    sink.clear();

    debug_trace(
        "runtime:replay:init",
        "replay initialized",
        &format!(
            "{{\"totalLines\":{},\"cols\":{},\"rows\":{}}}",
            state.doc.line_count(),
            cols,
            rows
        ),
    );

    let parsed_keys = parse_replay_keys(keys_str);

    for key in parsed_keys {
        let ctx = reducer_ctx(&state, &files, diff_ctx, cols, rows);
        let debug = gd_debug_enabled();
        let t_key_start = Instant::now();
        let result = handle_key(&mut state, key, &ctx);
        let key_dur = t_key_start.elapsed();

        match result {
            KeyResult::Quit => break,
            KeyResult::ReRender => {
                re_render(&mut state, &files, color, cols);
            }
            KeyResult::ReGenerate => {
                if !refresh_files_and_render(&mut files, &mut state, diff_ctx, color, cols) {
                    break;
                }
            }
            KeyResult::OpenEditor { .. } => {
                debug_trace("runtime:replay", "skipping OpenEditor in replay mode", "{}");
                continue;
            }
            KeyResult::ApplyPatch { .. } => {
                debug_trace("runtime:replay", "skipping ApplyPatch in replay mode", "{}");
                continue;
            }
            KeyResult::Continue => {}
        }

        sink.clear();
        let t_render_start = Instant::now();
        render_screen(&mut sink, &state, cols, rows);
        let render_dur = t_render_start.elapsed();

        if debug {
            trace_keystroke(key, key_dur, render_dur);
        }
    }

    debug_trace(
        "runtime:replay:done",
        "replay complete",
        &format_debug_state(&state),
    );
}
