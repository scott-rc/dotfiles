use std::io::{self, Write};
use std::time::Duration;

use crossterm::event::{self, Event, KeyEventKind};

use tui::pager::{
    ALT_SCREEN_OFF, ALT_SCREEN_ON, CURSOR_HIDE, CURSOR_SHOW, crossterm_to_key, get_term_size,
};

use crate::git::diff::DiffFile;
use crate::render::RenderOutput;

use super::reducer::handle_key;
use super::rendering::{content_height, render_screen};
use super::state::{DiffContext, capture_view_anchor, remap_after_document_swap};
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
    tui::pager::open_in_editor(path, line, false);
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
    let output = crate::render::render(files, width, color);
    let new_doc = Document::from_render_output(output);
    remap_after_document_swap(state, anchor, new_doc, files, cols as usize);

    debug_trace(
        "runtime:re_render",
        "post rerender state",
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

pub fn run_pager(output: RenderOutput, files: Vec<DiffFile>, color: bool, diff_ctx: &DiffContext) {
    let mut files = files;
    let mut stdout = io::BufWriter::new(io::stdout());

    let _ = write!(stdout, "{ALT_SCREEN_ON}{CURSOR_HIDE}");
    let _ = stdout.flush();
    let _ = crossterm::terminal::enable_raw_mode();

    let prev_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = crossterm::terminal::disable_raw_mode();
        let _ = io::stdout().write_all(format!("{CURSOR_SHOW}{ALT_SCREEN_OFF}").as_bytes());
        let _ = io::stdout().flush();
        prev_hook(info);
    }));

    let mut last_size = get_term_size();

    // Startup heuristic: decide full_context from initial diff characteristics
    let total_hunks: usize = files.iter().map(|f| f.hunks.len()).sum();
    let use_full_context = super::state::default_full_context(files.len(), total_hunks);

    // If full context is warranted, regenerate files with -U999999
    if use_full_context {
        files = regenerate_files(diff_ctx, true);
        if files.is_empty() {
            let _ = crossterm::terminal::disable_raw_mode();
            let _ = write!(stdout, "{CURSOR_SHOW}{ALT_SCREEN_OFF}");
            let _ = stdout.flush();
            return;
        }
    }

    let tree_entries = build_tree_entries(&files);
    let output = if use_full_context {
        // Re-render with full context files
        let width = last_size.0 as usize;
        crate::render::render(&files, width, color)
    } else {
        output
    };

    let mut state = PagerState::new(
        output.lines,
        output.line_map,
        output.file_starts,
        output.hunk_starts,
        tree_entries,
        last_size.0 as usize,
    );
    state.full_context = use_full_context;

    // Startup heuristic: decide view scope from rendered output size
    let view_scope = super::state::default_view_scope(
        files.len(),
        state.doc.line_count(),
        last_size.1 as usize,
    );
    state.view_scope = view_scope;

    re_render(&mut state, &files, color, last_size.0);
    render_screen(&mut stdout, &state, last_size.0, last_size.1);

    loop {
        let ev = match event::poll(Duration::from_millis(50)) {
            Ok(true) => match event::read() {
                Ok(ev) => ev,
                Err(_) => break,
            },
            Ok(false) => {
                let current_size = get_term_size();
                if current_size != last_size {
                    last_size = current_size;
                    re_render(&mut state, &files, color, last_size.0);
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
        let result = handle_key(&mut state, key, ch, last_size.1, last_size.0, &files, &diff_ctx.repo);
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
                re_render(&mut state, &files, color, last_size.0);
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
            }
            KeyResult::Continue => {}
        }
        render_screen(&mut stdout, &state, last_size.0, last_size.1);
    }

    let _ = crossterm::terminal::disable_raw_mode();
    let _ = write!(stdout, "{CURSOR_SHOW}{ALT_SCREEN_OFF}");
    let _ = stdout.flush();

    // Remove the custom panic hook (restores the default hook).
    let _ = std::panic::take_hook();
}
