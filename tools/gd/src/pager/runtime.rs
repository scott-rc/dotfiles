use std::io::{self, Write};
use std::time::{Duration, Instant, SystemTime};

use crossterm::event::{self, Event, KeyEventKind, KeyboardEnhancementFlags, PushKeyboardEnhancementFlags, PopKeyboardEnhancementFlags};

use tui::pager::{
    ALT_SCREEN_OFF, ALT_SCREEN_ON, CURSOR_HIDE, CURSOR_SHOW, Key, crossterm_to_key, get_term_size,
};

use crate::git::diff::DiffFile;

use super::reducer::handle_key;
use super::rendering::{content_height, render_screen};
use super::state::{DiffContext, ReducerCtx, capture_view_anchor, remap_after_document_swap};
use super::state::{Document, PagerState};
use super::tree::build_tree_entries;
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
    if full_context { cols.saturating_sub(1) } else { cols }
}

/// Compute the diff area width for the initial render, accounting for tree
/// panel and scrollbar. Avoids a redundant Phase 2 relayout after startup.
fn initial_render_width(cols: u16, tree_entries: &[super::tree::TreeEntry], file_count: usize, full_context: bool) -> usize {
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
fn trace_keystroke(
    key: Key,
    key_dur: Duration,
    render_dur: Duration,
) {
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

pub(crate) fn re_render(state: &mut PagerState, files: &[DiffFile], color: bool, cols: u16) {
    let anchor = capture_view_anchor(state);

    // Pre-resolve tree layout so diff_area_width uses the final tree_width.
    // Without this, the diff would be rendered at the old tree_width and then
    // remap_after_document_swap would recalculate tree_width, causing overflow.
    if !files.is_empty() {
        use super::tree::{build_tree_entries, compute_tree_width, resolve_tree_layout};
        let entries = build_tree_entries(files);
        let content_width = compute_tree_width(&entries);
        let has_directories = entries.iter().any(|e| e.file_idx.is_none());
        let file_count = state.doc.file_count().max(files.len());
        // Account for scrollbar column when full_context is active
        let effective_cols = if state.full_context {
            (cols as usize).saturating_sub(1)
        } else {
            cols as usize
        };
        if let Some(w) = resolve_tree_layout(content_width, effective_cols, has_directories, file_count) {
            if state.tree_visible {
                state.tree_width = w;
                state.tree_entries = entries;
            } else if !state.tree_user_hidden {
                // Auto-show: terminal widened enough for the tree
                state.tree_visible = true;
                state.tree_width = w;
                state.tree_entries = entries;
            }
        } else if state.tree_visible {
            state.tree_visible = false;
            state.tree_width = 0;
        }
    }

    let width = super::rendering::diff_area_width(
        cols,
        state.tree_width,
        state.tree_visible,
        state.full_context,
    );
    // Relayout: reuse Phase 1 styled content, only redo Phase 2 (wrapping/gutters)
    let styled_files = std::mem::take(&mut state.doc.styled_files);
    let output = crate::render::relayout(styled_files, width, color);
    let new_doc = Document::from_render_output(output);
    remap_after_document_swap(state, anchor, new_doc, files, cols as usize);

    debug_trace(
        "runtime:re_render",
        "post rerender state",
        &format_debug_state(state),
    );
}

/// Full render: re-style and relayout (for content changes).
pub(crate) fn full_render(state: &mut PagerState, files: &[DiffFile], color: bool, cols: u16) {
    let anchor = capture_view_anchor(state);

    // Pre-resolve tree layout so diff_area_width uses the final tree_width.
    if !files.is_empty() {
        use super::tree::{build_tree_entries, compute_tree_width, resolve_tree_layout};
        let entries = build_tree_entries(files);
        let content_width = compute_tree_width(&entries);
        let has_directories = entries.iter().any(|e| e.file_idx.is_none());
        let file_count = state.doc.file_count().max(files.len());
        let effective_cols = if state.full_context {
            (cols as usize).saturating_sub(1)
        } else {
            cols as usize
        };
        if let Some(w) = resolve_tree_layout(content_width, effective_cols, has_directories, file_count) {
            if state.tree_visible {
                state.tree_width = w;
                state.tree_entries = entries;
            } else if !state.tree_user_hidden {
                state.tree_visible = true;
                state.tree_width = w;
                state.tree_entries = entries;
            }
        } else if state.tree_visible {
            state.tree_visible = false;
            state.tree_width = 0;
        }
    }

    let width = super::rendering::diff_area_width(
        cols,
        state.tree_width,
        state.tree_visible,
        state.full_context,
    );
    let output = crate::render::render(files, width, color);
    let new_doc = Document::from_render_output(output);
    remap_after_document_swap(state, anchor, new_doc, files, cols as usize);

    debug_trace(
        "runtime:full_render",
        "post full_render state",
        &format_debug_state(state),
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

pub fn run_pager(files: Vec<DiffFile>, color: bool, use_full_context: bool, diff_ctx: &DiffContext) {
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

    // Pre-compute tree layout so we render at the final width upfront,
    // avoiding a redundant Phase 2 relayout.
    let tree_entries = build_tree_entries(&files);
    let effective_cols = effective_terminal_cols(last_size.0 as usize, use_full_context);
    let render_width = initial_render_width(last_size.0, &tree_entries, files.len(), use_full_context);

    let output = crate::render::render(&files, render_width, color);
    crate::debug::trace("pager", "post-render", t0);

    let doc = Document::from_render_output(output);
    let mut state = PagerState::from_doc(doc, tree_entries, effective_cols);
    state.full_context = use_full_context;

    // Startup heuristic: decide view scope from rendered output size
    let view_scope = super::state::default_view_scope(
        files.len(),
        state.doc.line_count(),
        last_size.1 as usize,
    );
    state.view_scope = view_scope;

    render_screen(&mut stdout, &state, last_size.0, last_size.1);

    debug_trace(
        "runtime:render_screen",
        "post initial render",
        &format!(
            "{{\"totalLines\":{}}}",
            state.doc.line_count()
        ),
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
                        files = regenerate_files(diff_ctx, state.full_context);
                        if files.is_empty() {
                            break;
                        }
                        full_render(&mut state, &files, color, last_size.0);
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

        let ch = content_height(last_size.1, &state);
        let ctx = ReducerCtx {
            content_height: ch,
            rows: last_size.1,
            cols: last_size.0,
            files: &files,
            repo: &diff_ctx.repo,
            source: &diff_ctx.source,
        };
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
                let base = format_debug_state(&state);
                debug_trace(
                    "runtime:run_pager:regenerate:before",
                    "regenerate start",
                    &format!(
                        "{},\"filesLen\":{}}}",
                        base.trim_end_matches('}'),
                        files.len()
                    ),
                );
                files = regenerate_files(diff_ctx, state.full_context);
                if files.is_empty() {
                    break;
                }
                full_render(&mut state, &files, color, last_size.0);
                last_index_mtime = git_index_mtime(&diff_ctx.repo);
                let base = format_debug_state(&state);
                debug_trace(
                    "runtime:run_pager:regenerate:after",
                    "regenerate complete",
                    &format!(
                        "{},\"filesLen\":{}}}",
                        base.trim_end_matches('}'),
                        files.len()
                    ),
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

                files = regenerate_files(diff_ctx, state.full_context);
                if files.is_empty() {
                    break;
                }
                full_render(&mut state, &files, color, last_size.0);
                last_index_mtime = git_index_mtime(&diff_ctx.repo);
            }
            KeyResult::ApplyPatch { patch, cached, reverse } => {
                let apply_result = if cached && !reverse {
                    crate::git::stage_patch(&diff_ctx.repo, &patch)
                } else if cached && reverse {
                    crate::git::unstage_patch(&diff_ctx.repo, &patch)
                } else {
                    crate::git::revert_patch(&diff_ctx.repo, &patch)
                };
                match apply_result {
                    Ok(()) => {
                        files = regenerate_files(diff_ctx, state.full_context);
                        if files.is_empty() {
                            break;
                        }
                        full_render(&mut state, &files, color, last_size.0);
                        last_index_mtime = git_index_mtime(&diff_ctx.repo);
                    }
                    Err(e) => {
                        state.status_message = format!(
                            "Apply failed: {}",
                            e.lines().next().unwrap_or(&e)
                        );
                    }
                }
            }
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

    // Pre-compute tree layout so we render at the final width upfront,
    // avoiding a redundant Phase 2 relayout.
    let tree_entries = build_tree_entries(&files);
    let effective_cols = effective_terminal_cols(cols as usize, use_full_context);
    let render_width = initial_render_width(cols, &tree_entries, files.len(), use_full_context);

    let output = crate::render::render(&files, render_width, color);
    crate::debug::trace("pager", "post-render", t0);

    let doc = Document::from_render_output(output);
    let mut state = PagerState::from_doc(doc, tree_entries, effective_cols);
    state.full_context = use_full_context;

    let view_scope = super::state::default_view_scope(
        files.len(),
        state.doc.line_count(),
        rows as usize,
    );
    state.view_scope = view_scope;

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
        let ch = content_height(rows, &state);
        let ctx = ReducerCtx {
            content_height: ch,
            rows,
            cols,
            files: &files,
            repo: &diff_ctx.repo,
            source: &diff_ctx.source,
        };
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
                files = regenerate_files(diff_ctx, state.full_context);
                if files.is_empty() {
                    break;
                }
                full_render(&mut state, &files, color, cols);
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
