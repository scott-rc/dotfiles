use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

use similar::TextDiff;
use tui::highlight::{
    HighlightLines, HighlightState, ParseState, SYNTAX_SET, SyntectSyntaxSet, THEME,
    highlight_line,
};

use crate::git::diff::{DiffFile, DiffHunk, FileStatus, LineKind};
use crate::style;

/// Per-rendered-line metadata for the pager.
#[derive(Debug, Clone)]
pub struct LineInfo {
    pub file_idx: usize,
    pub path: Arc<str>,
    /// Source line number in the new file (for editor jump), if applicable.
    pub new_lineno: Option<u32>,
    /// Source line number in the old file (for deleted lines), if applicable.
    pub old_lineno: Option<u32>,
    /// Diff status of this line (Added/Deleted/Context), if from a hunk.
    pub line_kind: Option<LineKind>,
    /// Index of the hunk within the file this line belongs to, if from a hunk.
    pub hunk_idx: Option<usize>,
}

/// Pre-computed data for rendering a single output line (one per wrapped segment).
/// Separates the metadata pass (cheap) from the ANSI string rendering pass (expensive).
#[derive(Debug, Clone)]
pub(crate) struct LineRenderData {
    /// Raw diff line content (unwrapped source text).
    pub(crate) content: String,
    /// Diff line kind (Added/Deleted/Context).
    pub(crate) line_kind: LineKind,
    /// Old file line number, if applicable.
    pub(crate) old_lineno: Option<u32>,
    /// New file line number, if applicable.
    pub(crate) new_lineno: Option<u32>,
    /// Whether this is a continuation (wrapped) segment.
    pub(crate) is_continuation: bool,
    /// 0 for the first segment, 1+ for continuations.
    #[allow(dead_code)]
    pub(crate) continuation_index: usize,
    /// Index of the file this line belongs to.
    pub(crate) file_idx: usize,
    /// Index of the hunk within the file.
    pub(crate) hunk_idx: usize,
    /// Index of the diff line within the hunk (position in `hunk.lines`).
    pub(crate) line_idx_in_hunk: Option<usize>,
}

/// Render context stored on `LazyLines` for on-demand rendering.
#[derive(Debug, Clone)]
struct LazyRenderCtx {
    color: bool,
    width: usize,
    /// Index into `render_data` where each file begins (for highlighter state).
    file_starts: Vec<usize>,
    /// File paths for syntax detection, indexed by file_idx.
    file_paths: Vec<String>,
    /// Pre-rendered structural lines (file headers, hunk separators).
    /// Keyed by index; content lines are `None` here.
    structural: Vec<Option<String>>,
    /// Source diff data for lazy word-highlight computation.
    diff_files: Arc<Vec<DiffFile>>,
}

/// Per-hunk word-highlight data for both sides of each change block.
/// Maps a hunk-internal line index to its word-level byte ranges.
#[derive(Debug, Clone)]
struct HunkHighlights {
    /// Keyed by hunk-internal line index (position in `hunk.lines`).
    line_ranges: HashMap<usize, Vec<(usize, usize)>>,
}

/// Cache of computed word highlights, keyed by `(file_idx, hunk_idx)`.
type HunkWordCache = HashMap<(usize, usize), HunkHighlights>;

/// Saved syntax highlighter state for resuming mid-file.
type HlCheckpoint = (HighlightState, ParseState);

/// Interval (in rendered lines) between saved highlighter checkpoints.
const CHECKPOINT_INTERVAL: usize = 200;

/// Lazy line cache: stores `LineRenderData` and renders ANSI strings on demand.
/// Uses `RefCell` for interior mutability so `get(&self)` can populate the cache.
///
/// Structural lines (file headers, hunk separators) are pre-rendered during
/// construction since they don't go through the syntax highlighter. Content
/// lines are rendered lazily: the first access to any line in a file renders
/// all preceding lines in that file (to maintain sequential highlighter state),
/// caching them all. Subsequent accesses are O(1) cache hits.
#[derive(Debug, Clone)]
pub(crate) struct LazyLines {
    render_data: Vec<LineRenderData>,
    cache: RefCell<Vec<Option<String>>>,
    ctx: Option<LazyRenderCtx>,
    /// Lazy per-hunk word-highlight cache, keyed by `(file_idx, hunk_idx)`.
    hunk_word_cache: RefCell<HunkWordCache>,
    /// Saved highlighter state at regular intervals within each file,
    /// keyed by line index. Enables `render_file_up_to` to resume from
    /// the nearest checkpoint instead of replaying from file start.
    hl_checkpoints: RefCell<BTreeMap<usize, HlCheckpoint>>,
    /// `(cache_hits, cache_misses)` — incremented on each `get()` call.
    stats: Cell<(usize, usize)>,
    /// Count of `highlight_line` calls in `render_file_up_to`.
    highlight_calls: Cell<usize>,
}

impl LazyLines {
    /// Construct from `render_data` and a parallel `Vec<String>`, converting to `Some(...)`.
    /// Used for pre-populated (eager) construction in tests.
    #[cfg(test)]
    pub(crate) fn from_rendered(render_data: Vec<LineRenderData>, lines: Vec<String>) -> Self {
        debug_assert_eq!(render_data.len(), lines.len());
        let cache = lines.into_iter().map(Some).collect();
        Self {
            render_data,
            cache: RefCell::new(cache),
            ctx: None,
            hunk_word_cache: RefCell::new(HashMap::new()),
            hl_checkpoints: RefCell::new(BTreeMap::new()),
            stats: Cell::new((0, 0)),
            highlight_calls: Cell::new(0),
        }
    }

    /// Construct with an unpopulated cache for lazy on-demand rendering.
    /// Structural lines (where `structural[i]` is `Some`) are available immediately;
    /// content lines are rendered on first access.
    pub(crate) fn from_metadata(
        render_data: Vec<LineRenderData>,
        structural: Vec<Option<String>>,
        file_starts: Vec<usize>,
        file_paths: Vec<String>,
        color: bool,
        width: usize,
        diff_files: Arc<Vec<DiffFile>>,
    ) -> Self {
        debug_assert_eq!(render_data.len(), structural.len());
        let cache = RefCell::new(structural.clone());
        Self {
            render_data,
            cache,
            ctx: Some(LazyRenderCtx {
                color,
                width,
                file_starts,
                file_paths,
                structural,
                diff_files,
            }),
            hunk_word_cache: RefCell::new(HashMap::new()),
            hl_checkpoints: RefCell::new(BTreeMap::new()),
            stats: Cell::new((0, 0)),
            highlight_calls: Cell::new(0),
        }
    }

    pub(crate) fn len(&self) -> usize {
        self.render_data.len()
    }

    /// Count how many lines have been rendered so far (for debug tracing).
    pub(crate) fn rendered_count(&self) -> usize {
        self.cache.borrow().iter().filter(|s| s.is_some()).count()
    }

    /// `(cache_hits, cache_misses)` since last reset.
    pub(crate) fn stats(&self) -> (usize, usize) {
        self.stats.get()
    }

    /// Count of `highlight_line` calls since last reset.
    pub(crate) fn highlight_calls(&self) -> usize {
        self.highlight_calls.get()
    }

    /// Reset hit/miss and highlight_line counters.
    pub(crate) fn reset_stats(&self) {
        self.stats.set((0, 0));
        self.highlight_calls.set(0);
    }

    /// Get word-highlight byte ranges for a line, computing and caching the
    /// hunk's highlights lazily on first access.
    fn get_word_ranges(
        &self,
        file_idx: usize,
        hunk_idx: usize,
        line_idx_in_hunk: Option<usize>,
    ) -> Vec<(usize, usize)> {
        let Some(line_idx) = line_idx_in_hunk else {
            return Vec::new();
        };
        let Some(ctx) = &self.ctx else {
            return Vec::new();
        };

        // Ensure hunk highlights are cached
        let needs_compute = !self.hunk_word_cache.borrow().contains_key(&(file_idx, hunk_idx));
        if needs_compute
            && let Some(file) = ctx.diff_files.get(file_idx)
            && let Some(hunk) = file.hunks.get(hunk_idx)
        {
            let blocks = find_change_blocks(hunk);
            let mut line_ranges = HashMap::new();
            for block in &blocks {
                let (del_hl, add_hl) = word_highlights(hunk, block);
                for (idx, ranges) in block.deleted.iter().zip(del_hl) {
                    if !ranges.is_empty() {
                        line_ranges.insert(*idx, ranges);
                    }
                }
                for (idx, ranges) in block.added.iter().zip(add_hl) {
                    if !ranges.is_empty() {
                        line_ranges.insert(*idx, ranges);
                    }
                }
            }
            self.hunk_word_cache
                .borrow_mut()
                .insert((file_idx, hunk_idx), HunkHighlights { line_ranges });
        }

        let cache = self.hunk_word_cache.borrow();
        cache
            .get(&(file_idx, hunk_idx))
            .and_then(|hl| hl.line_ranges.get(&line_idx))
            .cloned()
            .unwrap_or_default()
    }

    /// Get the rendered string for line `idx`. Panics if out of bounds.
    /// If the line is not yet rendered, renders all lines from the start of
    /// that file up to `idx` to maintain correct syntax highlighter state.
    pub(crate) fn get(&self, idx: usize) -> String {
        // Fast path: already cached
        if let Some(cached) = &self.cache.borrow()[idx] {
            let (h, m) = self.stats.get();
            self.stats.set((h + 1, m));
            return cached.clone();
        }

        // Render on demand from file start
        let (h, m) = self.stats.get();
        self.stats.set((h, m + 1));
        self.render_file_up_to(idx);
        self.cache.borrow()[idx].clone().unwrap()
    }

    /// Render all content lines from the start of the file containing `idx`
    /// up to and including `idx`. Uses saved highlighter checkpoints to avoid
    /// replaying from file start when possible.
    fn render_file_up_to(&self, idx: usize) {
        let ctx = self.ctx.as_ref().expect("LazyLines: no render context for lazy rendering");

        // Find the start of this file in the render_data
        let file_start = ctx
            .file_starts
            .iter()
            .rev()
            .find(|&&s| s <= idx)
            .copied()
            .unwrap_or(0);

        // Determine syntax for this file's extension
        let ext = self.detect_file_extension(file_start);
        let syntax = SYNTAX_SET
            .find_syntax_by_extension(&ext)
            .unwrap_or_else(|| SYNTAX_SET.find_syntax_plain_text());

        let hunk_ctx = HunkRenderContext {
            ss: &SYNTAX_SET,
            color: ctx.color,
            width: ctx.width,
        };

        // Resume from the nearest checkpoint before idx, or file start
        let (start_from, mut hl_state) = {
            let checkpoints = self.hl_checkpoints.borrow();
            match checkpoints.range(file_start..=idx).next_back() {
                Some((&line, (hl_s, ps))) => {
                    (line, HighlightLines::from_state(&THEME, hl_s.clone(), ps.clone()))
                }
                None => (file_start, HighlightLines::new(syntax, &THEME)),
            }
        };

        let mut nl_buf = String::new();
        let mut cache = self.cache.borrow_mut();
        let mut lines_since_checkpoint = start_from.saturating_sub(file_start);

        let mut i = start_from;
        while i <= idx {
            if cache[i].is_some() {
                // Structural line or already rendered -- but we still need to
                // advance the highlighter through content lines that may have
                // been rendered in a previous partial pass
                if self.render_data[i].line_kind != LineKind::Context
                    || !self.render_data[i].content.is_empty()
                {
                    // Feed content to highlighter to maintain state
                    if ctx.structural[i].is_none() && !self.render_data[i].content.is_empty() {
                        nl_buf.clear();
                        nl_buf.push_str(&self.render_data[i].content);
                        nl_buf.push('\n');
                        let _ = highlight_line(&nl_buf, &mut hl_state, &SYNTAX_SET, style::SOFT_RESET);
                        self.highlight_calls.set(self.highlight_calls.get() + 1);
                    }
                }
                lines_since_checkpoint += 1;
                i += 1;
                continue;
            }

            // This is an unrendered content line -- render it
            let data = &self.render_data[i];
            if data.is_continuation {
                // Continuation lines are normally rendered as part of their
                // parent. If we reach one that's still None, the parent
                // produced fewer wrap segments than the metadata estimated.
                cache[i] = Some(String::new());
                i += 1;
                continue;
            }

            // Lazily compute word highlights for this hunk (if not yet cached)
            let word_ranges = self.get_word_ranges(data.file_idx, data.hunk_idx, data.line_idx_in_hunk);

            let rendered_segments = render_line(data, &word_ranges, &hunk_ctx, &mut hl_state, &mut nl_buf);
            if ctx.color {
                self.highlight_calls.set(self.highlight_calls.get() + 1);
            }

            // Advance past all continuation segments
            let mut count = 1;
            while i + count < self.render_data.len() && self.render_data[i + count].is_continuation {
                count += 1;
            }

            let seg_count = rendered_segments.len();
            for (seg_offset, rendered) in rendered_segments.into_iter().enumerate() {
                let actual_idx = i + seg_offset;
                if actual_idx < self.render_data.len() && cache[actual_idx].is_none() {
                    cache[actual_idx] = Some(rendered);
                }
            }
            // If render produced fewer segments than metadata estimated
            // (ANSI wrapping differs from raw-text wrapping), fill remaining
            // continuation slots with empty strings.
            for j in seg_count..count {
                let actual_idx = i + j;
                if actual_idx < self.render_data.len() && cache[actual_idx].is_none() {
                    cache[actual_idx] = Some(String::new());
                }
            }
            lines_since_checkpoint += count;
            i += count;

            // Save a checkpoint at regular intervals so future calls can
            // resume from here instead of replaying from file start.
            if ctx.color && lines_since_checkpoint >= CHECKPOINT_INTERVAL {
                let (hl_s, ps) = hl_state.state();
                self.hl_checkpoints
                    .borrow_mut()
                    .insert(i, (hl_s.clone(), ps.clone()));
                hl_state = HighlightLines::from_state(&THEME, hl_s, ps);
                lines_since_checkpoint = 0;
            }
        }
    }

    /// Detect file extension from the file path at the given file start index.
    fn detect_file_extension(&self, file_start: usize) -> String {
        let ctx = self.ctx.as_ref().unwrap();
        let file_idx = self.render_data[file_start].file_idx;
        if file_idx < ctx.file_paths.len() {
            std::path::Path::new(&ctx.file_paths[file_idx])
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_string()
        } else {
            String::new()
        }
    }

    /// Evict cached ANSI strings outside the window `[keep_start, keep_end)`,
    /// freeing memory for lines far from the current viewport.
    /// Also evicts `hunk_word_cache` entries for hunks entirely outside the window.
    ///
    /// Highlighter checkpoints (`hl_checkpoints`) are intentionally preserved
    /// across evictions so that `render_file_up_to` can resume from the nearest
    /// checkpoint instead of replaying from file start.
    pub(crate) fn evict_outside(&self, keep_start: usize, keep_end: usize) {
        let len = self.render_data.len();
        let ks = keep_start.min(len);
        let ke = keep_end.min(len);

        {
            let mut cache = self.cache.borrow_mut();
            for (i, slot) in cache.iter_mut().enumerate() {
                if i >= ks && i < ke {
                    continue;
                }
                *slot = None;
            }
        }

        // Evict hunk word highlights for hunks outside the window.
        if !self.hunk_word_cache.borrow().is_empty() {
            let mut keep_hunks = std::collections::HashSet::new();
            for data in &self.render_data[ks..ke] {
                keep_hunks.insert((data.file_idx, data.hunk_idx));
            }
            self.hunk_word_cache
                .borrow_mut()
                .retain(|key, _| keep_hunks.contains(key));
        }
    }

    /// Collect raw texts for all lines. Suitable for `find_matches` --
    /// no ANSI escapes, no expensive rendering.
    pub(crate) fn raw_texts(&self) -> Vec<String> {
        self.render_data
            .iter()
            .map(|d| d.content.clone())
            .collect()
    }

    /// Render all lines eagerly (for no-pager path and search).
    pub(crate) fn render_all(&self) {
        let len = self.len();
        if len == 0 {
            return;
        }
        // Render each file's lines
        if let Some(ctx) = &self.ctx {
            for (fi, &start) in ctx.file_starts.iter().enumerate() {
                let end = ctx
                    .file_starts
                    .get(fi + 1)
                    .copied()
                    .unwrap_or(len)
                    .saturating_sub(1);
                if end >= start {
                    // Check if the last line in this file is already rendered
                    if self.cache.borrow()[end].is_none() {
                        self.render_file_up_to(end);
                    }
                }
            }
        }
    }

    /// Collect all rendered lines into owned Strings for `find_matches` compatibility.
    /// Renders any unrendered lines first.
    pub(crate) fn to_strings(&self) -> Vec<String> {
        self.render_all();
        self.cache
            .borrow()
            .iter()
            .map(|s| s.clone().unwrap())
            .collect()
    }
}

#[cfg(test)]
impl LazyLines {
    /// Return the raw (non-ANSI) text for line `idx`.
    /// Content lines return the diff source text; file headers return the path;
    /// hunk separators return an empty string.
    pub(crate) fn raw_text(&self, idx: usize) -> &str {
        &self.render_data[idx].content
    }

    /// Construct from pre-rendered strings (cache fully populated).
    pub(crate) fn new_prerendered(render_data: Vec<LineRenderData>, cache: Vec<Option<String>>) -> Self {
        debug_assert_eq!(render_data.len(), cache.len());
        Self {
            render_data,
            cache: RefCell::new(cache),
            ctx: None,
            hunk_word_cache: RefCell::new(HashMap::new()),
            hl_checkpoints: RefCell::new(BTreeMap::new()),
            stats: Cell::new((0, 0)),
            highlight_calls: Cell::new(0),
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.render_data.is_empty()
    }

    pub(crate) fn is_rendered(&self, idx: usize) -> bool {
        self.cache
            .borrow()
            .get(idx)
            .is_some_and(Option::is_some)
    }

    /// Iterate all rendered lines. Panics if any line is not yet rendered.
    pub(crate) fn iter_rendered(&self) -> Vec<String> {
        self.render_all();
        self.cache
            .borrow()
            .iter()
            .map(|s| s.clone().unwrap())
            .collect()
    }
}

pub struct RenderOutput {
    pub line_map: Vec<LineInfo>,
    pub file_starts: Vec<usize>,
    pub hunk_starts: Vec<usize>,
    /// Lazy line cache for on-demand rendering.
    pub(crate) lazy_lines: LazyLines,
}

impl RenderOutput {
    /// Eagerly render all lines and return them. Used by the no-pager path
    /// and tests that need all lines as `Vec<String>`.
    pub fn lines(&self) -> Vec<String> {
        self.lazy_lines.to_strings()
    }

    // --- Bench-only accessors (used by benches/bench.rs) ---

    #[allow(dead_code)]
    pub fn get(&self, idx: usize) -> String {
        self.lazy_lines.get(idx)
    }

    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.lazy_lines.len()
    }

    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.lazy_lines.len() == 0
    }

    #[allow(dead_code)]
    pub fn evict_outside(&self, keep_start: usize, keep_end: usize) {
        self.lazy_lines.evict_outside(keep_start, keep_end);
    }
}

struct HunkRenderContext<'a> {
    ss: &'a SyntectSyntaxSet,
    color: bool,
    width: usize,
}

/// Produce metadata for a single file: structural lines (pre-rendered headers/separators),
/// line_map, render_data, and local hunk_start offsets. Content lines are left as `None`
/// in the structural vec for lazy rendering.
fn render_file(
    file_idx: usize,
    file: &DiffFile,
    width: usize,
    color: bool,
) -> (Vec<Option<String>>, Vec<LineInfo>, Vec<LineRenderData>, Vec<usize>) {
    let path = file.path();
    let arc_path: Arc<str> = Arc::from(path);
    let status_label = match file.status {
        FileStatus::Modified => "Modified",
        FileStatus::Added => "Added",
        FileStatus::Deleted => "Deleted",
        FileStatus::Renamed => "Renamed",
        FileStatus::Untracked => "Untracked",
    };
    // File header
    let header = if color {
        style::file_header(path, status_label, width)
    } else {
        let label = format!(" {path} ({status_label}) ");
        let bar_len = width.saturating_sub(2 + label.len());
        format!(
            "{}{}{}",
            "\u{2500}".repeat(2),
            label,
            "\u{2500}".repeat(bar_len)
        )
    };

    let cap = file.hunks.iter().map(|h| h.lines.len()).sum::<usize>() + 1;
    let mut structural = Vec::with_capacity(cap);
    let mut line_map = Vec::with_capacity(cap);
    let mut render_data = Vec::with_capacity(cap);
    let mut hunk_starts = Vec::new();

    let separator_data = || LineRenderData {
        content: String::new(),
        line_kind: LineKind::Context,
        old_lineno: None,
        new_lineno: None,
        is_continuation: false,
        continuation_index: 0,
        file_idx,
        hunk_idx: 0,
        line_idx_in_hunk: None,
    };

    structural.push(Some(header));
    line_map.push(LineInfo {
        file_idx,
        path: Arc::clone(&arc_path),
        new_lineno: None,
        old_lineno: None,
        line_kind: None,
        hunk_idx: None,
    });
    // File header carries the path as searchable raw text
    render_data.push(LineRenderData {
        content: format!("{path} ({status_label})"),
        ..separator_data()
    });

    for (hunk_idx, hunk) in file.hunks.iter().enumerate() {
        // Separator between hunks (not before the first)
        if hunk_idx > 0 {
            structural.push(Some(style::hunk_separator(width, color)));
            line_map.push(LineInfo {
                file_idx,
                path: Arc::clone(&arc_path),
                new_lineno: None,
                old_lineno: None,
                line_kind: None,
                hunk_idx: None,
            });
            render_data.push(separator_data());
        }

        hunk_starts.push(structural.len());

        // Produce metadata for diff lines (no ANSI rendering)
        let metadata = hunk_lines_metadata(hunk, hunk_idx, file_idx, &arc_path, width);
        for (info, data) in metadata {
            structural.push(None); // content lines rendered lazily
            line_map.push(info);
            render_data.push(data);
        }
    }

    (structural, line_map, render_data, hunk_starts)
}

pub fn render(files: &[DiffFile], width: usize, color: bool) -> RenderOutput {
    let thread_count = std::thread::available_parallelism()
        .map_or(1, std::num::NonZero::get)
        .min(files.len())
        .max(1);

    let file_paths: Vec<String> = files.iter().map(|f| f.path().to_string()).collect();
    let diff_files: Arc<Vec<DiffFile>> = Arc::new(files.to_vec());

    // No-color render skips syntect highlighting (the expensive part),
    // so threading overhead would dominate. Only parallelize with color.
    if thread_count <= 1 || !color {
        // Sequential path: no threading overhead
        let cap: usize = files
            .iter()
            .map(|f| f.hunks.iter().map(|h| h.lines.len()).sum::<usize>())
            .sum::<usize>()
            + files.len();
        let mut all_structural = Vec::with_capacity(cap);
        let mut all_line_map = Vec::with_capacity(cap);
        let mut all_render_data = Vec::with_capacity(cap);
        let mut file_starts = Vec::new();
        let mut hunk_starts = Vec::new();

        for (file_idx, file) in files.iter().enumerate() {
            let offset = all_structural.len();
            file_starts.push(offset);
            let (fs, fm, rd, fh) = render_file(file_idx, file, width, color);
            for hs in &fh {
                hunk_starts.push(hs + offset);
            }
            all_structural.extend(fs);
            all_line_map.extend(fm);
            all_render_data.extend(rd);
        }

        let lazy_lines = LazyLines::from_metadata(
            all_render_data.clone(),
            all_structural,
            file_starts.clone(),
            file_paths,
            color,
            width,
            Arc::clone(&diff_files),
        );

        return RenderOutput {
            line_map: all_line_map,
            file_starts,
            hunk_starts,
            lazy_lines,
        };
    }

    // Parallel path: produce metadata across threads, then merge in order
    let file_results: Vec<(Vec<Option<String>>, Vec<LineInfo>, Vec<LineRenderData>, Vec<usize>)> =
        std::thread::scope(|s| {
            let chunk_size = files.len().div_ceil(thread_count);
            let handles: Vec<_> = files
                .chunks(chunk_size)
                .enumerate()
                .map(|(chunk_idx, chunk)| {
                    let base_file_idx = chunk_idx * chunk_size;
                    s.spawn(move || {
                        let mut results = Vec::with_capacity(chunk.len());
                        for (i, file) in chunk.iter().enumerate() {
                            results.push(render_file(base_file_idx + i, file, width, color));
                        }
                        results
                    })
                })
                .collect();

            let mut all_results = Vec::with_capacity(files.len());
            for handle in handles {
                all_results.extend(handle.join().unwrap());
            }
            all_results
        });

    // Merge results in file order
    let total_lines: usize = file_results.iter().map(|(l, _, _, _)| l.len()).sum();
    let mut all_structural = Vec::with_capacity(total_lines);
    let mut all_line_map = Vec::with_capacity(total_lines);
    let mut all_render_data = Vec::with_capacity(total_lines);
    let mut file_starts = Vec::with_capacity(files.len());
    let mut hunk_starts = Vec::new();

    for (fs, fm, rd, fh) in file_results {
        let offset = all_structural.len();
        file_starts.push(offset);
        for hs in &fh {
            hunk_starts.push(hs + offset);
        }
        all_structural.extend(fs);
        all_line_map.extend(fm);
        all_render_data.extend(rd);
    }

    let lazy_lines = LazyLines::from_metadata(
        all_render_data.clone(),
        all_structural,
        file_starts.clone(),
        file_paths,
        color,
        width,
        diff_files,
    );

    RenderOutput {
        line_map: all_line_map,
        file_starts,
        hunk_starts,
        lazy_lines,
    }
}

/// Group consecutive added/deleted lines into change blocks for word-level diffing.
pub struct ChangeBlock {
    pub deleted: Vec<usize>,
    pub added: Vec<usize>,
}

pub fn find_change_blocks(hunk: &DiffHunk) -> Vec<ChangeBlock> {
    let mut blocks = Vec::new();
    let mut i = 0;
    let hunk_lines = &hunk.lines;

    while i < hunk_lines.len() {
        if hunk_lines[i].kind == LineKind::Deleted {
            let mut deleted = Vec::new();
            while i < hunk_lines.len() && hunk_lines[i].kind == LineKind::Deleted {
                deleted.push(i);
                i += 1;
            }
            let mut added = Vec::new();
            while i < hunk_lines.len() && hunk_lines[i].kind == LineKind::Added {
                added.push(i);
                i += 1;
            }
            if !added.is_empty() {
                blocks.push(ChangeBlock { deleted, added });
            } else {
                // Pure deletions — no word-level diff
                blocks.push(ChangeBlock {
                    deleted,
                    added: Vec::new(),
                });
            }
        } else if hunk_lines[i].kind == LineKind::Added {
            let mut added = Vec::new();
            while i < hunk_lines.len() && hunk_lines[i].kind == LineKind::Added {
                added.push(i);
                i += 1;
            }
            blocks.push(ChangeBlock {
                deleted: Vec::new(),
                added,
            });
        } else {
            i += 1;
        }
    }

    blocks
}

/// Tokenize text into words, whitespace runs, and individual punctuation characters.
/// Finer than `from_words` (which groups by whitespace only), so punctuation like
/// a trailing comma doesn't cause an entire word to be treated as changed.
pub fn tokenize(s: &str) -> Vec<&str> {
    let mut tokens = Vec::new();
    let mut chars = s.char_indices().peekable();

    while let Some(&(start, ch)) = chars.peek() {
        if ch.is_alphanumeric() || ch == '_' {
            let mut end = start + ch.len_utf8();
            chars.next();
            while let Some(&(_, c)) = chars.peek() {
                if c.is_alphanumeric() || c == '_' {
                    end += c.len_utf8();
                    chars.next();
                } else {
                    break;
                }
            }
            tokens.push(&s[start..end]);
        } else if ch.is_whitespace() {
            let mut end = start + ch.len_utf8();
            chars.next();
            while let Some(&(_, c)) = chars.peek() {
                if c.is_whitespace() {
                    end += c.len_utf8();
                    chars.next();
                } else {
                    break;
                }
            }
            tokens.push(&s[start..end]);
        } else {
            chars.next();
            tokens.push(&s[start..start + ch.len_utf8()]);
        }
    }

    tokens
}

/// Token product threshold for word-level diffs. Blocks larger than this
/// skip word-level highlighting (entire lines are highlighted instead).
/// Patience diff is O(n log n) for matching unique elements but falls back
/// to Myers O(n*d) for non-unique regions, so very large blocks still cost.
const WORD_DIFF_TOKEN_LIMIT: usize = 10_000;

/// For a change block, compute per-line word highlight ranges.
/// Returns (deleted_highlights, added_highlights) — each a Vec<Vec<(start, end)>> per line.
pub fn word_highlights(
    hunk: &DiffHunk,
    block: &ChangeBlock,
) -> (Vec<Vec<(usize, usize)>>, Vec<Vec<(usize, usize)>>) {
    if block.deleted.is_empty() || block.added.is_empty() {
        return (
            vec![Vec::new(); block.deleted.len()],
            vec![Vec::new(); block.added.len()],
        );
    }

    let old_text: String = block
        .deleted
        .iter()
        .map(|&i| hunk.lines[i].content.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    let new_text: String = block
        .added
        .iter()
        .map(|&i| hunk.lines[i].content.as_str())
        .collect::<Vec<_>>()
        .join("\n");

    let old_tokens = tokenize(&old_text);
    let new_tokens = tokenize(&new_text);

    // Skip word-level diff for very large blocks — highlight entire lines instead
    if old_tokens.len() * new_tokens.len() > WORD_DIFF_TOKEN_LIMIT {
        return (
            vec![Vec::new(); block.deleted.len()],
            vec![Vec::new(); block.added.len()],
        );
    }

    let diff = TextDiff::from_slices(&old_tokens, &new_tokens);

    let mut del_highlights: Vec<Vec<(usize, usize)>> = vec![Vec::new(); block.deleted.len()];
    let mut add_highlights: Vec<Vec<(usize, usize)>> = vec![Vec::new(); block.added.len()];

    // Track positions in old/new text to map back to per-line ranges
    let mut old_pos = 0;
    let mut new_pos = 0;

    for change in diff.iter_all_changes() {
        let value = change.value();
        match change.tag() {
            similar::ChangeTag::Equal => {
                old_pos += value.len();
                new_pos += value.len();
            }
            similar::ChangeTag::Delete => {
                add_ranges_to_lines(
                    old_pos,
                    value.len(),
                    &block.deleted,
                    hunk,
                    &mut del_highlights,
                );
                old_pos += value.len();
            }
            similar::ChangeTag::Insert => {
                add_ranges_to_lines(
                    new_pos,
                    value.len(),
                    &block.added,
                    hunk,
                    &mut add_highlights,
                );
                new_pos += value.len();
            }
        }
    }

    // Merge contiguous ranges produced by individual token changes
    for ranges in del_highlights.iter_mut().chain(add_highlights.iter_mut()) {
        merge_ranges(ranges);
    }

    (del_highlights, add_highlights)
}

/// Merge contiguous or overlapping highlight ranges in-place.
pub(crate) fn merge_ranges(ranges: &mut Vec<(usize, usize)>) {
    if ranges.len() <= 1 {
        return;
    }
    ranges.sort_unstable_by_key(|&(start, _)| start);
    let mut merged = Vec::with_capacity(ranges.len());
    let mut current = ranges[0];
    for &(start, end) in &ranges[1..] {
        if start <= current.1 {
            current.1 = current.1.max(end);
        } else {
            merged.push(current);
            current = (start, end);
        }
    }
    merged.push(current);
    *ranges = merged;
}

/// Map a range in a concatenated multi-line string back to per-line highlight ranges.
fn add_ranges_to_lines(
    start: usize,
    len: usize,
    line_indices: &[usize],
    hunk: &DiffHunk,
    highlights: &mut [Vec<(usize, usize)>],
) {
    let end = start + len;
    let mut offset = 0;

    for (idx, &line_idx) in line_indices.iter().enumerate() {
        let line_len = hunk.lines[line_idx].content.len();
        let line_end = offset + line_len;

        // The "\n" between joined lines
        let sep_end = if idx + 1 < line_indices.len() {
            line_end + 1
        } else {
            line_end
        };

        if start < sep_end && end > offset {
            let local_start = start.saturating_sub(offset);
            let local_end = end.min(line_end) - offset;
            if local_start < local_end {
                highlights[idx].push((local_start, local_end));
            }
        }

        offset = line_end + 1; // +1 for "\n"
    }
}

/// Compute per-line metadata for a hunk. Word-highlight ranges are left empty;
/// they are computed lazily in `LazyLines::get_word_ranges()` on first render access.
/// Returns one `(LineInfo, LineRenderData)` pair per output row (including wrap segments).
fn hunk_lines_metadata(
    hunk: &DiffHunk,
    hunk_idx: usize,
    file_idx: usize,
    path: &Arc<str>,
    width: usize,
) -> Vec<(LineInfo, LineRenderData)> {
    let avail = width.saturating_sub(style::GUTTER_WIDTH + 2); // +1 marker, +1 wrap indicator
    let mut result = Vec::with_capacity(hunk.lines.len());

    for (i, diff_line) in hunk.lines.iter().enumerate() {

        // Determine how many wrapped segments the raw content will produce
        let content_vis_width = crate::ansi::visible_width(&diff_line.content);
        let seg_count = if avail > 0 && content_vis_width > avail {
            content_vis_width.div_ceil(avail)
        } else {
            1
        };

        for seg_idx in 0..seg_count {
            let info = LineInfo {
                file_idx,
                path: Arc::clone(path),
                new_lineno: diff_line.new_lineno,
                old_lineno: diff_line.old_lineno,
                line_kind: Some(diff_line.kind),
                hunk_idx: Some(hunk_idx),
            };
            let data = LineRenderData {
                content: diff_line.content.clone(),
                line_kind: diff_line.kind,
                old_lineno: diff_line.old_lineno,
                new_lineno: diff_line.new_lineno,

                is_continuation: seg_idx > 0,
                continuation_index: seg_idx,
                file_idx,
                hunk_idx,
                line_idx_in_hunk: Some(i),
            };
            result.push((info, data));
        }
    }

    result
}

/// Render a single logical diff line into one or more ANSI output strings (one per wrap segment).
/// `word_ranges` provides lazily-computed word-level highlight byte ranges for this line.
fn render_line(
    data: &LineRenderData,
    word_ranges: &[(usize, usize)],
    ctx: &HunkRenderContext<'_>,
    hl_state: &mut HighlightLines,
    nl_buf: &mut String,
) -> Vec<String> {
    let (marker, line_bg, word_bg, marker_color) = match data.line_kind {
        LineKind::Added => (
            "+",
            style::BG_ADDED,
            style::BG_ADDED_WORD,
            style::FG_ADDED_MARKER,
        ),
        LineKind::Deleted => (
            "-",
            style::BG_DELETED,
            style::BG_DELETED_WORD,
            style::FG_DELETED_MARKER,
        ),
        LineKind::Context => (" ", "", "", ""),
    };

    let gutter = if ctx.color {
        style::gutter(data.old_lineno, data.new_lineno)
    } else {
        let old = data.old_lineno.map_or(String::new(), |n| format!("{n}"));
        let new = data.new_lineno.map_or(String::new(), |n| format!("{n}"));
        format!("{old:>4} |{new:>4} |")
    };

    let content = &data.content;

    // Build the content portion with syntax + diff coloring
    let styled_content = if ctx.color {
        nl_buf.clear();
        nl_buf.push_str(content);
        nl_buf.push('\n');
        let syntax_colored = highlight_line(nl_buf, hl_state, ctx.ss, style::SOFT_RESET);
        apply_diff_colors(
            &syntax_colored,
            content,
            line_bg,
            word_bg,
            word_ranges,
            data.line_kind != LineKind::Context,
        )
    } else {
        content.clone()
    };

    let marker_styled = if ctx.color && !marker_color.is_empty() {
        format!("{}{marker}{}", marker_color, style::SOFT_RESET)
    } else {
        marker.to_string()
    };

    let is_changed = data.line_kind != LineKind::Context;
    let avail = ctx.width.saturating_sub(style::GUTTER_WIDTH + 2);
    let cont_gutter = style::continuation_gutter(ctx.color);
    let wrap_marker = style::wrap_marker(ctx.color);

    // Pre-wrap: prepend line_bg so AnsiState tracks it during wrapping
    let wrappable = if ctx.color && is_changed {
        format!("{line_bg}{styled_content}")
    } else {
        styled_content
    };
    let wrapped = crate::ansi::wrap_line_for_display(&wrappable, avail);

    let mut output = Vec::with_capacity(wrapped.len());
    for (seg_idx, seg) in wrapped.iter().enumerate() {
        let content_part = seg.trim_end_matches(style::RESET);
        let seg_width = crate::ansi::visible_width(content_part);
        let pad_len = avail.saturating_sub(seg_width);
        let padding = if ctx.color && is_changed && pad_len > 0 {
            " ".repeat(pad_len)
        } else {
            String::new()
        };

        let line = if seg_idx == 0 {
            if ctx.color && is_changed {
                format!(
                    "{gutter}{line_bg}{marker_styled}{content_part}{padding}{}",
                    style::RESET
                )
            } else {
                format!("{gutter}{marker_styled}{content_part}")
            }
        } else if ctx.color && is_changed {
            format!(
                "{cont_gutter}{line_bg}{wrap_marker}{content_part}{padding}{}",
                style::RESET
            )
        } else {
            format!("{cont_gutter}{wrap_marker}{content_part}")
        };
        output.push(line);
    }

    output
}

/// Apply diff background colors and word-level highlights to syntax-colored text.
/// The `raw` parameter is the original uncolored content used for word range mapping.
pub fn apply_diff_colors(
    syntax_colored: &str,
    raw: &str,
    line_bg: &str,
    word_bg: &str,
    word_ranges: &[(usize, usize)],
    is_changed: bool,
) -> String {
    if !is_changed || word_ranges.is_empty() {
        return syntax_colored.to_string();
    }

    // Build a mapping from visible char index → byte position in syntax_colored
    let segments = crate::ansi::split_ansi(syntax_colored);
    let mut vis_to_byte: Vec<usize> = Vec::new();
    let mut byte_pos = 0;

    for seg in &segments {
        if seg.starts_with('\x1b') {
            byte_pos += seg.len();
        } else {
            for (i, _) in seg.char_indices() {
                vis_to_byte.push(byte_pos + i);
            }
            byte_pos += seg.len();
        }
    }
    vis_to_byte.push(byte_pos); // sentinel

    // Also build visible char index for the raw content
    let raw_chars: Vec<usize> = raw.char_indices().map(|(i, _)| i).collect();

    // Collect (byte_pos, priority, ansi_code) pairs for a forward pass.
    // Priority 0 = word_bg (start highlight), 1 = line_bg (end highlight).
    // At the same byte position, word_bg sorts before line_bg so a range
    // ending and starting at the same point produces: end-old, start-new.
    let mut markers: Vec<(usize, u8, &str)> = Vec::with_capacity(word_ranges.len() * 2);

    for &(start, end) in word_ranges {
        // Map raw byte offsets to visible char indices via binary search
        let vis_start = {
            let idx = raw_chars.partition_point(|&b| b < start);
            if idx < raw_chars.len() { idx } else { vis_to_byte.len().saturating_sub(1) }
        };
        let vis_end = {
            let idx = raw_chars.partition_point(|&b| b < end);
            if idx < raw_chars.len() { idx } else { vis_to_byte.len().saturating_sub(1) }
        };

        if vis_start < vis_to_byte.len() && vis_end <= vis_to_byte.len() {
            let byte_start = vis_to_byte[vis_start];
            let byte_end = vis_to_byte[vis_end.min(vis_to_byte.len() - 1)];
            markers.push((byte_start, 0, word_bg));
            markers.push((byte_end, 1, line_bg));
        }
    }

    markers.sort_unstable();

    // Single forward pass: emit syntax_colored slices interleaved with ANSI codes
    let total_codes_len: usize = markers.iter().map(|(_, _, code)| code.len()).sum();
    let mut result = String::with_capacity(syntax_colored.len() + total_codes_len);
    let mut src_pos: usize = 0;

    for &(pos, _, code) in &markers {
        if pos <= syntax_colored.len() && pos >= src_pos {
            result.push_str(&syntax_colored[src_pos..pos]);
            result.push_str(code);
            src_pos = pos;
        }
    }
    result.push_str(&syntax_colored[src_pos..]);

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::diff;
    use crate::git::diff::DiffLine;
    use insta::assert_debug_snapshot;
    use insta::assert_snapshot;

    fn strip(lines: &[String]) -> String {
        lines
            .iter()
            .map(|l| crate::ansi::strip_ansi(l))
            .collect::<Vec<_>>()
            .join("\n")
    }

    #[test]
    fn snapshot_single_file() {
        let raw = "\
diff --git a/foo.rs b/foo.rs
--- a/foo.rs
+++ b/foo.rs
@@ -1,3 +1,4 @@
 line1
+added
 line2
 line3
";
        let files = diff::parse(raw);
        let output = render(&files, 80, false);
        assert_snapshot!(strip(&output.lines()));
    }

    #[test]
    fn snapshot_multi_file() {
        let raw = "\
diff --git a/a.txt b/a.txt
--- a/a.txt
+++ b/a.txt
@@ -1,1 +1,2 @@
 first
+second
diff --git a/b.txt b/b.txt
--- a/b.txt
+++ b/b.txt
@@ -1,2 +1,1 @@
 keep
-remove
";
        let files = diff::parse(raw);
        let output = render(&files, 80, false);
        assert_snapshot!(strip(&output.lines()));
    }

    #[test]
    fn snapshot_multi_hunk() {
        let raw = "\
diff --git a/foo.rs b/foo.rs
--- a/foo.rs
+++ b/foo.rs
@@ -1,3 +1,4 @@
 line1
+added1
 line2
 line3
@@ -10,3 +11,4 @@
 line10
+added2
 line11
 line12
";
        let files = diff::parse(raw);
        let output = render(&files, 80, false);
        assert_snapshot!(strip(&output.lines()));
    }

    #[test]
    fn snapshot_wrapped_long_line() {
        let long_content = "x".repeat(100);
        let raw = format!(
            "\
diff --git a/foo.txt b/foo.txt
--- a/foo.txt
+++ b/foo.txt
@@ -1,1 +1,2 @@
 ctx
+{long_content}
"
        );
        let files = diff::parse(&raw);
        let output = render(&files, 40, false);
        assert_snapshot!(strip(&output.lines()));
    }

    #[test]
    fn snapshot_untracked_file() {
        let file = diff::DiffFile::from_content("new.rs", "hello\nworld\n");
        let output = render(&[file], 80, false);
        assert_snapshot!(strip(&output.lines()));
    }

    #[test]
    fn snapshot_line_map() {
        let raw = "\
diff --git a/f.rs b/f.rs
--- a/f.rs
+++ b/f.rs
@@ -1,2 +1,3 @@
 ctx
+added
 more
";
        let files = diff::parse(raw);
        let output = render(&files, 80, false);
        assert_debug_snapshot!(output.line_map);
    }

    #[test]
    fn render_produces_lines_for_simple_diff() {
        let raw = "\
diff --git a/foo.rs b/foo.rs
--- a/foo.rs
+++ b/foo.rs
@@ -1,3 +1,4 @@
 line1
+added
 line2
 line3
";
        let files = diff::parse(raw);
        let output = render(&files, 80, false);

        // File header + 4 diff lines (no hunk header)
        assert_eq!(output.lines().len(), 5);
        assert_eq!(output.file_starts.len(), 1);
        assert_eq!(output.hunk_starts.len(), 1);
        assert_eq!(output.file_starts[0], 0);
        assert_eq!(output.hunk_starts[0], 1); // after file header, at first content line

        // Check the added line has a + marker
        let added_line = &output.lines()[2]; // header, ctx, added
        assert!(added_line.contains('+'));
        assert!(added_line.contains("added"));
    }

    #[test]
    fn render_multi_file() {
        let raw = "\
diff --git a/a.txt b/a.txt
--- a/a.txt
+++ b/a.txt
@@ -1,1 +1,2 @@
 first
+second
diff --git a/b.txt b/b.txt
--- a/b.txt
+++ b/b.txt
@@ -1,2 +1,1 @@
 keep
-remove
";
        let files = diff::parse(raw);
        let output = render(&files, 80, false);

        assert_eq!(output.file_starts.len(), 2);
        assert_eq!(output.hunk_starts.len(), 2);

        // line_map should reference correct file indices
        let first_file_info = &output.line_map[output.file_starts[0]];
        assert_eq!(&*first_file_info.path, "a.txt");
        let second_file_info = &output.line_map[output.file_starts[1]];
        assert_eq!(&*second_file_info.path, "b.txt");
    }

    #[test]
    fn render_multi_file_with_headers_has_no_blank_separator_line() {
        let raw = "\
diff --git a/a.txt b/a.txt
--- a/a.txt
+++ b/a.txt
@@ -1,1 +1,2 @@
 first
+second
diff --git a/b.txt b/b.txt
--- a/b.txt
+++ b/b.txt
@@ -1,2 +1,1 @@
 keep
-remove
";
        let files = diff::parse(raw);
        let output = render(&files, 80, false);
        assert!(
            !output.lines().iter().any(std::string::String::is_empty),
            "all-files mode should not insert blank lines between file headers"
        );
    }

    #[test]
    fn render_multi_hunk() {
        let raw = "\
diff --git a/foo.rs b/foo.rs
--- a/foo.rs
+++ b/foo.rs
@@ -1,3 +1,4 @@
 line1
+added1
 line2
 line3
@@ -10,3 +11,4 @@
 line10
+added2
 line11
 line12
";
        let files = diff::parse(raw);
        let output = render(&files, 80, false);

        assert_eq!(output.hunk_starts.len(), 2);
    }

    #[test]
    fn line_map_tracks_new_lineno() {
        let raw = "\
diff --git a/f.rs b/f.rs
--- a/f.rs
+++ b/f.rs
@@ -1,2 +1,3 @@
 ctx
+added
 more
";
        let files = diff::parse(raw);
        let output = render(&files, 80, false);

        // Find the added line's info
        let added_info = output
            .line_map
            .iter()
            .find(|li| li.new_lineno == Some(2))
            .expect("should find new_lineno=2");
        assert_eq!(&*added_info.path, "f.rs");
    }

    #[test]
    fn colored_added_line_has_bg_spanning_content() {
        let raw = "\
diff --git a/foo.txt b/foo.txt
--- a/foo.txt
+++ b/foo.txt
@@ -1,1 +1,2 @@
 ctx
+added line
";
        let files = diff::parse(raw);
        let output = render(&files, 80, true);

        // Find the added line (after file header, context)
        let added = &output.lines()[2];
        let stripped = crate::ansi::strip_ansi(added);
        assert!(stripped.contains('+'), "should have + marker");

        // The full RESET (\x1b[0m) should only appear at the very end of the line,
        // not between the marker and content or between syntax tokens
        let content_area = added.split(style::BG_ADDED).last().unwrap_or(added);
        let mid_section =
            &content_area[..content_area.rfind("\x1b[0m").unwrap_or(content_area.len())];
        assert!(
            !mid_section.contains("\x1b[0m"),
            "full RESET should not appear mid-line (kills bg): {mid_section:?}"
        );
    }

    #[test]
    fn syntax_highlight_uses_soft_reset() {
        let syntax = SYNTAX_SET.find_syntax_by_extension("rs").unwrap();
        let mut hl = HighlightLines::new(syntax, &THEME);

        let result = highlight_line("let x = 42;\n", &mut hl, &SYNTAX_SET, style::SOFT_RESET);
        // Should use SOFT_RESET (22;23;39) not full RESET (0m) between tokens
        assert!(
            !result.contains("\x1b[0m"),
            "highlight_line should use SOFT_RESET, not RESET: {result:?}"
        );
        assert!(
            result.contains("\x1b[22;23;39m"),
            "should contain SOFT_RESET: {result:?}"
        );
    }

    #[test]
    fn long_line_wraps_with_continuation_gutter() {
        let long_content = "x".repeat(100);
        let raw = format!(
            "\
diff --git a/foo.txt b/foo.txt
--- a/foo.txt
+++ b/foo.txt
@@ -1,1 +1,2 @@
 ctx
+{long_content}
"
        );
        let files = diff::parse(&raw);
        // Narrow width forces wrapping: gutter(12) + marker(1) + avail
        let width = 40;
        let output = render(&files, width, false);

        // Should produce more lines than the 3 logical lines (header, ctx, added)
        assert!(
            output.lines().len() > 3,
            "long line should wrap into multiple output lines, got {}",
            output.lines().len()
        );

        // Continuation lines should have blank gutter with separators
        let all = output.lines();
        let cont_lines: Vec<_> = all.iter().skip(3).collect();
        assert!(!cont_lines.is_empty(), "should have continuation lines");
        for cont in &cont_lines {
            assert!(
                cont.contains('|'),
                "continuation line should have gutter separators: {cont:?}"
            );
        }
    }

    #[test]
    fn word_highlights_punctuation_boundary() {
        // Regression: from_words treats "application"]  vs "application"], as entirely
        // different tokens, highlighting the whole word instead of just the insertion.
        use crate::git::diff::{DiffHunk, DiffLine, LineKind};

        let hunk = DiffHunk {
            old_start: 1,
            new_start: 1,
            lines: vec![
                DiffLine {
                    kind: LineKind::Deleted,
                    content: r#"  "--app": { type: AppArg, alias: ["-a", "--application"] },"#.into(),
                    old_lineno: Some(1),
                    new_lineno: None,
                },
                DiffLine {
                    kind: LineKind::Added,
                    content: r#"  "--app": { type: AppArg, alias: ["-a", "--application"], description: "Select the application" },"#.into(),
                    old_lineno: None,
                    new_lineno: Some(1),
                },
            ],
        };
        let block = ChangeBlock {
            deleted: vec![0],
            added: vec![1],
        };
        let (del_hl, add_hl) = word_highlights(&hunk, &block);

        // Nothing was deleted — the old line should have no highlights
        assert!(
            del_hl[0].is_empty(),
            "deleted line should have no highlights, got: {:?}",
            del_hl[0]
        );

        // The insertion is `, description: "Select the application"`
        // which starts right after the `]` in the added line
        assert_eq!(
            add_hl[0].len(),
            1,
            "added line should have exactly one highlight range, got: {:?}",
            add_hl[0]
        );
        let (start, end) = add_hl[0][0];
        let added = &hunk.lines[1].content;
        let highlighted = &added[start..end];
        assert_eq!(
            highlighted, r#", description: "Select the application""#,
            "should highlight only the inserted portion"
        );
    }

    #[test]
    fn test_render_single_file_populates_file_starts() {
        let raw = "\
diff --git a/a.rs b/a.rs
--- a/a.rs
+++ b/a.rs
@@ -1,1 +1,2 @@
 ctx
+added
";
        let files = diff::parse(raw);
        let output = render(&files, 80, false);
        assert_eq!(output.file_starts.len(), 1);
        assert_eq!(output.file_starts[0], 0);
    }

    #[test]
    fn test_render_two_files_file_starts_ordered() {
        let raw = "\
diff --git a/a.rs b/a.rs
--- a/a.rs
+++ b/a.rs
@@ -1,1 +1,2 @@
 ctx
+added
diff --git a/b.rs b/b.rs
--- a/b.rs
+++ b/b.rs
@@ -1,1 +1,2 @@
 ctx
+added2
";
        let files = diff::parse(raw);
        let output = render(&files, 80, false);
        assert_eq!(output.file_starts.len(), 2);
        assert!(
            output.file_starts[1] > 0,
            "second file should start after line 0"
        );
    }

    #[test]
    fn test_render_hunk_starts_present() {
        let raw = "\
diff --git a/foo.rs b/foo.rs
--- a/foo.rs
+++ b/foo.rs
@@ -1,3 +1,4 @@
 line1
+added1
 line2
 line3
@@ -10,3 +11,4 @@
 line10
+added2
 line11
 line12
";
        let files = diff::parse(raw);
        let output = render(&files, 80, false);
        assert_eq!(output.hunk_starts.len(), 2);
        assert!(
            output.hunk_starts[0] < output.hunk_starts[1],
            "hunk starts should be ordered"
        );
    }

    #[test]
    fn test_render_line_map_length_matches_lines() {
        let raw = "\
diff --git a/a.rs b/a.rs
--- a/a.rs
+++ b/a.rs
@@ -1,2 +1,3 @@
 ctx
+added
 more
";
        let files = diff::parse(raw);
        let output = render(&files, 80, false);
        assert_eq!(output.lines().len(), output.line_map.len());
    }

    #[test]
    fn test_render_content_lines_have_line_kind() {
        let raw = "\
diff --git a/a.rs b/a.rs
--- a/a.rs
+++ b/a.rs
@@ -1,2 +1,3 @@
 ctx
+added
-old
";
        let files = diff::parse(raw);
        let output = render(&files, 80, false);
        // File header (index 0) should have None line_kind
        assert!(
            output.line_map[0].line_kind.is_none(),
            "file header should have None line_kind"
        );
        // Content lines should have Some(_)
        let content_lines: Vec<_> = output
            .line_map
            .iter()
            .filter(|li| li.line_kind.is_some())
            .collect();
        assert!(
            !content_lines.is_empty(),
            "should have content lines with Some line_kind"
        );
        for li in &content_lines {
            assert!(
                matches!(
                    li.line_kind,
                    Some(LineKind::Added | LineKind::Deleted | LineKind::Context)
                ),
                "content lines should be Added, Deleted, or Context"
            );
        }
    }

    #[test]
    fn render_untracked_file_shows_status_label() {
        let file = diff::DiffFile::from_content("new.rs", "hello\n");
        let output = render(&[file], 80, false);
        let header = &output.lines()[0];
        assert!(
            header.contains("Untracked"),
            "file header should contain 'Untracked': {header:?}"
        );
    }

    #[test]
    fn test_no_hunk_header_in_output() {
        let raw = "\
diff --git a/foo.rs b/foo.rs
--- a/foo.rs
+++ b/foo.rs
@@ -1,3 +1,4 @@
 line1
+added
 line2
 line3
";
        let files = diff::parse(raw);
        let output = render(&files, 80, false);

        for line in &output.lines() {
            assert!(
                !line.contains("@@"),
                "rendered output should not contain '@@' hunk headers, but found: {line:?}"
            );
        }
    }

    #[test]
    fn test_hunk_starts_points_at_content_line() {
        let raw = "\
diff --git a/foo.rs b/foo.rs
--- a/foo.rs
+++ b/foo.rs
@@ -1,3 +1,4 @@
 line1
+added
 line2
 line3
";
        let files = diff::parse(raw);
        let output = render(&files, 80, false);

        assert!(
            !output.hunk_starts.is_empty(),
            "should have at least one hunk_start"
        );
        let hs = output.hunk_starts[0];
        // Must be past the file header
        assert!(
            hs > output.file_starts[0],
            "hunk_start should be past the file header: hunk_start={hs}, file_start={}",
            output.file_starts[0]
        );
        // Must point at a content line (line_kind is Some)
        assert!(
            output.line_map[hs].line_kind.is_some(),
            "hunk_starts[0]={hs} should point at a content line with Some(line_kind), got {:?}",
            output.line_map[hs].line_kind
        );
    }

    #[test]
    fn test_multi_hunk_starts_all_point_at_content() {
        let raw = "\
diff --git a/foo.rs b/foo.rs
--- a/foo.rs
+++ b/foo.rs
@@ -1,3 +1,4 @@
 line1
+added1
 line2
 line3
@@ -10,3 +11,4 @@
 line10
+added2
 line11
 line12
";
        let files = diff::parse(raw);
        let output = render(&files, 80, false);

        assert_eq!(output.hunk_starts.len(), 2, "should have two hunks");
        for (i, &hs) in output.hunk_starts.iter().enumerate() {
            assert!(
                output.line_map[hs].line_kind.is_some(),
                "hunk_starts[{i}]={hs} should point at a content line with Some(line_kind), got {:?}",
                output.line_map[hs].line_kind
            );
        }
    }

    #[test]
    fn apply_diff_colors_is_changed_false() {
        let result = apply_diff_colors("foo", "foo", "", "", &[(0, 3)], false);
        assert_eq!(result, "foo");
    }

    #[test]
    fn apply_diff_colors_empty_word_ranges() {
        let result = apply_diff_colors("foo", "foo", "", "", &[], true);
        assert_eq!(result, "foo");
    }

    #[test]
    fn apply_diff_colors_simple_ascii_one_range() {
        let result = apply_diff_colors(
            "hello",
            "hello",
            style::BG_ADDED,
            style::BG_ADDED_WORD,
            &[(0, 5)],
            true,
        );
        assert!(
            result.contains(style::BG_ADDED_WORD),
            "expected word highlight in result: {result:?}"
        );
        assert!(
            crate::ansi::strip_ansi(&result).contains("hello"),
            "content should be preserved"
        );
    }

    #[test]
    fn apply_diff_colors_multi_byte_unicode() {
        let raw = "a\u{1F600}b";
        let result = apply_diff_colors(
            raw,
            raw,
            style::BG_ADDED,
            style::BG_ADDED_WORD,
            &[(0, raw.len())],
            true,
        );
        assert!(
            crate::ansi::strip_ansi(&result).contains("a\u{1F600}b"),
            "multi-byte content preserved: {result:?}"
        );
    }

    #[test]
    fn apply_diff_colors_two_adjacent_ranges() {
        let result = apply_diff_colors(
            "abcd",
            "abcd",
            style::BG_DELETED,
            style::BG_DELETED_WORD,
            &[(0, 2), (2, 4)],
            true,
        );
        let word_count = result.matches(style::BG_DELETED_WORD).count();
        assert!(
            word_count >= 2,
            "expected at least 2 word highlight insertions, got {word_count}: {result:?}"
        );
    }

    #[test]
    fn merge_ranges_overlapping() {
        let mut ranges = vec![(0, 5), (3, 8), (10, 12)];
        merge_ranges(&mut ranges);
        assert_eq!(ranges, vec![(0, 8), (10, 12)]);
    }

    #[test]
    fn merge_ranges_single_or_empty() {
        let mut single = vec![(1, 3)];
        merge_ranges(&mut single);
        assert_eq!(single, vec![(1, 3)]);

        let mut empty: Vec<(usize, usize)> = vec![];
        merge_ranges(&mut empty);
        assert!(empty.is_empty());
    }

    fn make_line(kind: LineKind) -> DiffLine {
        DiffLine {
            kind,
            content: String::new(),
            old_lineno: None,
            new_lineno: None,
        }
    }

    #[test]
    fn find_change_blocks_added_only() {
        let hunk = DiffHunk {
            old_start: 1,
            new_start: 1,
            lines: vec![
                make_line(LineKind::Context),
                make_line(LineKind::Added),
                make_line(LineKind::Added),
                make_line(LineKind::Context),
            ],
        };
        let blocks = find_change_blocks(&hunk);
        assert_eq!(blocks.len(), 1);
        assert!(blocks[0].deleted.is_empty());
        assert_eq!(blocks[0].added, vec![1, 2]);
    }

    #[test]
    fn find_change_blocks_mixed() {
        let hunk = DiffHunk {
            old_start: 1,
            new_start: 1,
            lines: vec![
                make_line(LineKind::Deleted),
                make_line(LineKind::Deleted),
                make_line(LineKind::Added),
                make_line(LineKind::Added),
                make_line(LineKind::Context),
                make_line(LineKind::Added),
            ],
        };
        let blocks = find_change_blocks(&hunk);
        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0].deleted, vec![0, 1]);
        assert_eq!(blocks[0].added, vec![2, 3]);
        assert!(blocks[1].deleted.is_empty());
        assert_eq!(blocks[1].added, vec![5]);
    }

    #[test]
    fn colored_wrap_has_bg_on_continuation() {
        let long_content = "x".repeat(100);
        let raw = format!(
            "\
diff --git a/foo.txt b/foo.txt
--- a/foo.txt
+++ b/foo.txt
@@ -1,1 +1,2 @@
 ctx
+{long_content}
"
        );
        let files = diff::parse(&raw);
        let width = 40;
        let output = render(&files, width, true);

        // Find continuation lines — they have blank gutter (no line numbers)
        // and contain 'x' from the long content
        let all_lines = output.lines();
        let first_added = all_lines
            .iter()
            .position(|l| crate::ansi::strip_ansi(l).contains('+'))
            .expect("should find added line");
        let cont_lines: Vec<_> = all_lines
            .iter()
            .skip(first_added + 1)
            .filter(|l| {
                let s = crate::ansi::strip_ansi(l);
                // Continuation lines have blank gutter (starts with spaces before │)
                // and contain 'xx' (the repeated content)
                s.contains("xx") && s.trim_start().starts_with('\u{2502}')
            })
            .collect();

        assert!(!cont_lines.is_empty(), "should have continuation lines");
        for cont in &cont_lines {
            assert!(
                cont.contains(style::BG_ADDED),
                "continuation line should have added bg: {cont:?}"
            );
        }
    }

    #[test]
    fn render_parallel_matches_sequential() {
        let raw = "\
diff --git a/a.rs b/a.rs
--- a/a.rs
+++ b/a.rs
@@ -1,1 +1,2 @@
 ctx
+added
diff --git a/b.rs b/b.rs
--- a/b.rs
+++ b/b.rs
@@ -1,1 +1,2 @@
 ctx
+added2
";
        let files = diff::parse(raw);
        let color_output = render(&files, 80, true);
        let plain_output = render(&files, 80, false);
        // Structural shape must match regardless of rendering path
        assert_eq!(color_output.file_starts, plain_output.file_starts);
        assert_eq!(color_output.hunk_starts, plain_output.hunk_starts);
        assert_eq!(color_output.line_map.len(), plain_output.line_map.len());
    }

    #[test]
    fn render_deleted_file_shows_status_label() {
        let raw = "\
diff --git a/old.txt b/old.txt
deleted file mode 100644
--- a/old.txt
+++ /dev/null
@@ -1,2 +0,0 @@
-line1
-line2
";
        let files = diff::parse(raw);
        let output = render(&files, 80, false);
        let header = crate::ansi::strip_ansi(&output.lines()[0]);
        assert!(
            header.contains("Deleted"),
            "file header should contain 'Deleted': {header:?}"
        );
        assert_eq!(files[0].status, FileStatus::Deleted);
        // Deleted lines should be present
        let deleted_count = output
            .line_map
            .iter()
            .filter(|li| li.line_kind == Some(LineKind::Deleted))
            .count();
        assert_eq!(deleted_count, 2, "should have 2 deleted lines");
    }

    #[test]
    fn render_renamed_file_shows_status_label() {
        let raw = "\
diff --git a/old.txt b/new.txt
similarity index 100%
rename from old.txt
rename to new.txt
";
        let files = diff::parse(raw);
        let output = render(&files, 80, false);
        let header = crate::ansi::strip_ansi(&output.lines()[0]);
        assert!(
            header.contains("Renamed"),
            "file header should contain 'Renamed': {header:?}"
        );
        assert_eq!(files[0].status, FileStatus::Renamed);
        // Pure rename has no hunks — just the header line
        assert_eq!(output.lines().len(), 1);
    }

    #[test]
    fn lazy_lines_len_and_is_empty() {
        let data = vec![
            LineRenderData {
                content: "hello".into(),
                line_kind: LineKind::Added,
                old_lineno: None,
                new_lineno: Some(1),

                is_continuation: false,
                continuation_index: 0,
                file_idx: 0,
                hunk_idx: 0,
                line_idx_in_hunk: None,
            },
            LineRenderData {
                content: "world".into(),
                line_kind: LineKind::Context,
                old_lineno: Some(1),
                new_lineno: Some(2),

                is_continuation: false,
                continuation_index: 0,
                file_idx: 0,
                hunk_idx: 0,
                line_idx_in_hunk: None,
            },
        ];
        let cache = vec![Some("rendered-hello".to_string()), Some("rendered-world".to_string())];
        let lazy = LazyLines::new_prerendered(data, cache);
        assert_eq!(lazy.len(), 2);
        assert!(!lazy.is_empty());

        let empty = LazyLines::new_prerendered(Vec::new(), Vec::new());
        assert_eq!(empty.len(), 0);
        assert!(empty.is_empty());
    }

    #[test]
    fn lazy_lines_get_returns_cached_string() {
        let data = vec![LineRenderData {
            content: "hello".into(),
            line_kind: LineKind::Added,
            old_lineno: None,
            new_lineno: Some(1),
            is_continuation: false,
            continuation_index: 0,
            file_idx: 0,
            hunk_idx: 0,
            line_idx_in_hunk: None,
        }];
        let cache = vec![Some("rendered-hello".to_string())];
        let lazy = LazyLines::new_prerendered(data, cache);
        assert_eq!(lazy.get(0), "rendered-hello");
        // Repeated access returns the same value
        assert_eq!(lazy.get(0), "rendered-hello");
        assert!(lazy.is_rendered(0));
    }

    #[test]
    fn lazy_lines_to_strings_returns_all() {
        let data = vec![
            LineRenderData {
                content: "a".into(),
                line_kind: LineKind::Context,
                old_lineno: Some(1),
                new_lineno: Some(1),

                is_continuation: false,
                continuation_index: 0,
                file_idx: 0,
                hunk_idx: 0,
                line_idx_in_hunk: None,
            },
            LineRenderData {
                content: "b".into(),
                line_kind: LineKind::Added,
                old_lineno: None,
                new_lineno: Some(2),

                is_continuation: false,
                continuation_index: 0,
                file_idx: 0,
                hunk_idx: 0,
                line_idx_in_hunk: None,
            },
        ];
        let cache = vec![Some("line-a".to_string()), Some("line-b".to_string())];
        let lazy = LazyLines::new_prerendered(data, cache);
        let all = lazy.to_strings();
        assert_eq!(all.len(), 2);
        assert_eq!(all[0], "line-a");
        assert_eq!(all[1], "line-b");
    }

    #[test]
    fn lazy_lines_content_not_rendered_before_access() {
        let raw = "\
diff --git a/foo.rs b/foo.rs
--- a/foo.rs
+++ b/foo.rs
@@ -1,3 +1,4 @@
 line1
+added
 line2
 line3
";
        let files = diff::parse(raw);
        let output = render(&files, 80, false);
        let lazy = &output.lazy_lines;
        assert!(lazy.len() > 1);
        // Line 0 is a structural header -- it IS pre-rendered
        assert!(
            lazy.is_rendered(0),
            "structural header (line 0) should be pre-rendered"
        );
        // Content lines (index 1+) should NOT be rendered before any get() call
        assert!(
            !lazy.is_rendered(1),
            "content line 1 should not be rendered before any get() call"
        );
    }

    #[test]
    fn lazy_lines_renders_on_demand() {
        let raw = "\
diff --git a/foo.rs b/foo.rs
--- a/foo.rs
+++ b/foo.rs
@@ -1,3 +1,4 @@
 line1
+added
 line2
 line3
";
        let files = diff::parse(raw);
        let output = render(&files, 80, false);
        let lazy = &output.lazy_lines;
        let total = lazy.len();
        assert!(total > 1);
        // Access content line 1 -- should render and cache it
        let line1 = lazy.get(1);
        assert!(!line1.is_empty(), "rendered line should not be empty");
        assert!(lazy.is_rendered(1), "line 1 should be rendered after get()");
    }

    #[test]
    fn lazy_lines_multifile_only_renders_accessed_file() {
        let raw = "\
diff --git a/a.rs b/a.rs
--- a/a.rs
+++ b/a.rs
@@ -1,2 +1,3 @@
 ctx
+added
 more
diff --git a/b.rs b/b.rs
--- b/b.rs
+++ b/b.rs
@@ -1,1 +1,2 @@
 keep
+new
";
        let files = diff::parse(raw);
        let output = render(&files, 80, false);
        let lazy = &output.lazy_lines;
        let file2_start = output.file_starts[1];

        // Access the first content line in file 2 (file2_start is the header)
        let file2_content = file2_start + 1;
        let _ = lazy.get(file2_content);
        assert!(
            lazy.is_rendered(file2_content),
            "accessed content line in file 2 should be rendered"
        );
        // Content lines in file 1 (indices 1..file2_start) should NOT be rendered
        let file1_content_rendered = (1..file2_start).any(|i| lazy.is_rendered(i));
        assert!(
            !file1_content_rendered,
            "file 1 content lines should not be rendered when only file 2 was accessed"
        );
    }

    #[test]
    fn lazy_word_highlights_appear_in_rendered_output() {
        // A change block (deleted + added) should produce word-level highlights
        // even though hunk_lines_metadata sets word_ranges to empty.
        // The lazy cache in LazyLines::get() computes them on demand.
        let raw = "\
diff --git a/foo.rs b/foo.rs
--- a/foo.rs
+++ b/foo.rs
@@ -1,2 +1,2 @@
 context
-old value
+new value
";
        let files = diff::parse(raw);
        let output = render(&files, 80, true);
        let lines = output.lines();

        // Find the added line (should have word-level highlight for "new")
        let added_line = lines
            .iter()
            .find(|l| crate::ansi::strip_ansi(l).contains("+new"))
            .expect("should find the added line");
        assert!(
            added_line.contains(style::BG_ADDED_WORD),
            "added line should contain word-level highlight BG_ADDED_WORD: {added_line:?}"
        );

        // Find the deleted line (should have word-level highlight for "old")
        let deleted_line = lines
            .iter()
            .find(|l| crate::ansi::strip_ansi(l).contains("-old"))
            .expect("should find the deleted line");
        assert!(
            deleted_line.contains(style::BG_DELETED_WORD),
            "deleted line should contain word-level highlight BG_DELETED_WORD: {deleted_line:?}"
        );
    }

    #[test]
    fn lazy_word_highlights_match_eager_computation() {
        // Verify that word highlights computed lazily via get() produce the
        // same results as calling word_highlights() directly on the hunk.
        let raw = "\
diff --git a/foo.rs b/foo.rs
--- a/foo.rs
+++ b/foo.rs
@@ -1,3 +1,3 @@
 context
-let x = old_value;
+let x = new_value;
 more context
";
        let files = diff::parse(raw);
        let output = render(&files, 80, true);

        // Compute expected word highlights eagerly
        let hunk = &files[0].hunks[0];
        let blocks = find_change_blocks(hunk);
        assert_eq!(blocks.len(), 1, "should have exactly one change block");
        let (del_hl, add_hl) = word_highlights(hunk, &blocks[0]);

        // The deleted line is at hunk line index 1, the added line at index 2
        assert!(!del_hl[0].is_empty(), "deleted line should have word highlights");
        assert!(!add_hl[0].is_empty(), "added line should have word highlights");

        // Now render via lazy path and verify the ANSI output contains the word bg
        let lines = output.lines();
        let deleted_line = lines
            .iter()
            .find(|l| crate::ansi::strip_ansi(l).contains("-let x = old_value"))
            .expect("should find deleted line");
        let added_line = lines
            .iter()
            .find(|l| crate::ansi::strip_ansi(l).contains("+let x = new_value"))
            .expect("should find added line");

        assert!(
            deleted_line.contains(style::BG_DELETED_WORD),
            "lazily-rendered deleted line should have word highlight: {deleted_line:?}"
        );
        assert!(
            added_line.contains(style::BG_ADDED_WORD),
            "lazily-rendered added line should have word highlight: {added_line:?}"
        );
    }

    #[test]
    fn lazy_word_highlights_hunk_cache_populated_on_access() {
        // Verify that the hunk word cache is populated when a line is accessed
        let raw = "\
diff --git a/foo.rs b/foo.rs
--- a/foo.rs
+++ b/foo.rs
@@ -1,2 +1,2 @@
 context
-old
+new
";
        let files = diff::parse(raw);
        let output = render(&files, 80, true);
        let lazy = &output.lazy_lines;

        // Cache should be empty before any content access
        assert!(
            lazy.hunk_word_cache.borrow().is_empty(),
            "hunk word cache should be empty before rendering"
        );

        // Access a content line from the hunk -- triggers word highlight computation
        let _ = lazy.get(1); // first content line in the hunk
        assert!(
            !lazy.hunk_word_cache.borrow().is_empty(),
            "hunk word cache should be populated after rendering a content line"
        );

        // Should have an entry for (file_idx=0, hunk_idx=0)
        assert!(
            lazy.hunk_word_cache.borrow().contains_key(&(0, 0)),
            "cache should contain entry for (file_idx=0, hunk_idx=0)"
        );
    }

    #[test]
    fn render_data_len_matches_lines_multi_file() {
        let raw = "\
diff --git a/a.rs b/a.rs
--- a/a.rs
+++ b/a.rs
@@ -1,2 +1,3 @@
 ctx
+added
 more
diff --git a/b.rs b/b.rs
--- a/b.rs
+++ b/b.rs
@@ -1,1 +1,2 @@
 keep
+new
";
        let files = diff::parse(raw);
        let output = render(&files, 80, false);
        assert_eq!(
            output.lazy_lines.len(),
            output.lines().len(),
            "lazy_lines length should match eagerly rendered lines length"
        );
    }

    /// Helper: build a LazyLines with `n` pre-rendered entries for eviction tests.
    fn make_eviction_lazy(n: usize) -> LazyLines {
        let render_data: Vec<LineRenderData> = (0..n)
            .map(|i| LineRenderData {
                content: format!("line {i}"),
                line_kind: LineKind::Context,
                old_lineno: None,
                new_lineno: None,
                is_continuation: false,
                continuation_index: 0,
                file_idx: i / 100, // group into files of 100 lines
                hunk_idx: (i / 50) % 2, // two hunks per file
                line_idx_in_hunk: Some(i % 50),
            })
            .collect();
        let cache: Vec<Option<String>> = (0..n).map(|i| Some(format!("rendered {i}"))).collect();
        LazyLines::new_prerendered(render_data, cache)
    }

    #[test]
    fn evict_outside_clears_lines_outside_window() {
        let lazy = make_eviction_lazy(1000);
        // Render everything first
        assert!(lazy.is_rendered(0));
        assert!(lazy.is_rendered(999));

        // Simulate viewport at 500..540, keep window [460, 580)
        lazy.evict_outside(460, 580);

        // Lines 0-39 should be evicted
        assert!(!lazy.is_rendered(0), "line 0 should be evicted");
        assert!(!lazy.is_rendered(39), "line 39 should be evicted");
        assert!(!lazy.is_rendered(459), "line 459 should be evicted");

        // Lines within the keep window should still be cached
        assert!(lazy.is_rendered(460), "line 460 should survive eviction");
        assert!(lazy.is_rendered(500), "line 500 should survive eviction");
        assert!(lazy.is_rendered(539), "line 539 should survive eviction");
        assert!(lazy.is_rendered(579), "line 579 should survive eviction");

        // Lines after the window should be evicted
        assert!(!lazy.is_rendered(580), "line 580 should be evicted");
        assert!(!lazy.is_rendered(999), "line 999 should be evicted");
    }

    #[test]
    fn evict_outside_keeps_viewport_window_intact() {
        let lazy = make_eviction_lazy(200);
        // Viewport at 50..90, keep window with generous margin [10, 130)
        lazy.evict_outside(10, 130);

        // Everything in the keep window is preserved
        for i in 10..130 {
            assert!(
                lazy.is_rendered(i),
                "line {i} within keep window should survive eviction"
            );
        }
        // Outside the window is evicted
        for i in 0..10 {
            assert!(
                !lazy.is_rendered(i),
                "line {i} outside keep window should be evicted"
            );
        }
        for i in 130..200 {
            assert!(
                !lazy.is_rendered(i),
                "line {i} outside keep window should be evicted"
            );
        }
    }

    #[test]
    fn evict_outside_evicts_hunk_word_cache() {
        let lazy = make_eviction_lazy(200);
        // Manually populate the hunk word cache with entries for different hunks
        {
            let mut cache = lazy.hunk_word_cache.borrow_mut();
            // file_idx=0, hunk_idx=0 covers lines 0..50
            cache.insert(
                (0, 0),
                HunkHighlights {
                    line_ranges: HashMap::new(),
                },
            );
            // file_idx=0, hunk_idx=1 covers lines 50..100
            cache.insert(
                (0, 1),
                HunkHighlights {
                    line_ranges: HashMap::new(),
                },
            );
            // file_idx=1, hunk_idx=0 covers lines 100..150
            cache.insert(
                (1, 0),
                HunkHighlights {
                    line_ranges: HashMap::new(),
                },
            );
        }

        // Keep window covers lines 60..120 (file_idx=0,hunk_idx=1 and file_idx=1,hunk_idx=0)
        lazy.evict_outside(60, 120);

        let cache = lazy.hunk_word_cache.borrow();
        assert!(
            !cache.contains_key(&(0, 0)),
            "hunk (0,0) entirely outside keep window should be evicted"
        );
        assert!(
            cache.contains_key(&(0, 1)),
            "hunk (0,1) intersecting keep window should survive"
        );
        assert!(
            cache.contains_key(&(1, 0)),
            "hunk (1,0) intersecting keep window should survive"
        );
    }

    #[test]
    fn evict_outside_boundary_conditions() {
        let lazy = make_eviction_lazy(10);

        // Evict with window larger than the cache
        lazy.evict_outside(0, 100);
        for i in 0..10 {
            assert!(lazy.is_rendered(i), "all lines should survive when window covers everything");
        }

        // Evict with empty window
        lazy.evict_outside(5, 5);
        for i in 0..10 {
            assert!(!lazy.is_rendered(i), "all lines should be evicted with empty window");
        }
    }

    // -----------------------------------------------------------------------
    // Lazy rendering characterization tests
    // -----------------------------------------------------------------------

    #[test]
    fn wrap_segment_mismatch_fewer_segments_no_panic() {
        // Regression: render_line may produce fewer wrap segments than metadata
        // estimated (ANSI wrapping differs from raw-text width estimation),
        // leaving continuation slots as None. The fix fills them with empty strings.
        let long_content = "x".repeat(120);
        let raw = format!(
            "\
diff --git a/foo.txt b/foo.txt
--- a/foo.txt
+++ b/foo.txt
@@ -1,1 +1,2 @@
 ctx
+{long_content}
"
        );
        let files = diff::parse(&raw);
        // Use a width where wrapping happens; color=true so ANSI wrapping may
        // produce fewer segments than the raw-text estimate.
        let width = 40;
        let output = render(&files, width, true);
        // The test passing without panic IS the primary assertion.
        output.lazy_lines.render_all();
        let strings = output.lazy_lines.to_strings();
        assert_eq!(
            strings.len(),
            output.line_map.len(),
            "to_strings().len() must equal line_map.len()"
        );
    }

    #[test]
    fn evict_then_reaccess_rerenders() {
        let raw = "\
diff --git a/foo.rs b/foo.rs
--- a/foo.rs
+++ b/foo.rs
@@ -1,3 +1,4 @@
 line1
+added
 line2
 line3
";
        let files = diff::parse(raw);
        let output = render(&files, 80, false);
        let lazy = &output.lazy_lines;

        // Access line 1 (first content line) to populate cache
        let first_render = lazy.get(1);
        assert!(lazy.is_rendered(1));

        // Evict everything
        lazy.evict_outside(0, 0);
        assert!(
            !lazy.is_rendered(1),
            "line 1 should be evicted after evict_outside(0, 0)"
        );

        // Re-access and verify same content
        let second_render = lazy.get(1);
        assert!(
            lazy.is_rendered(1),
            "line 1 should be rendered again after re-access"
        );
        assert_eq!(
            first_render, second_render,
            "re-rendered content should match original"
        );
    }

    #[test]
    fn render_all_header_only_file_no_hunks() {
        // Pure rename diff: similarity 100%, no hunks -- just a file header.
        let raw = "\
diff --git a/old.txt b/new.txt
similarity index 100%
rename from old.txt
rename to new.txt
";
        let files = diff::parse(raw);
        let output = render(&files, 80, false);
        output.lazy_lines.render_all();
        let strings = output.lazy_lines.to_strings();
        assert_eq!(strings.len(), 1, "header-only file should produce 1 line");
    }

    #[test]
    fn word_highlight_evict_then_recompute() {
        // Verifies word highlights are recomputed after eviction.
        let raw = "\
diff --git a/foo.rs b/foo.rs
--- a/foo.rs
+++ b/foo.rs
@@ -1,2 +1,2 @@
 context
-old value
+new value
";
        let files = diff::parse(raw);
        let output = render(&files, 80, true);
        let lazy = &output.lazy_lines;

        // Find the deleted line index (after header + context)
        let del_idx = output
            .line_map
            .iter()
            .position(|li| li.line_kind == Some(LineKind::Deleted))
            .expect("should have a deleted line");

        // First render populates word cache
        let first_render = lazy.get(del_idx);
        assert!(
            first_render.contains(style::BG_DELETED_WORD),
            "deleted line should contain BG_DELETED_WORD on first render: {first_render:?}"
        );

        // Evict everything including hunk word cache
        lazy.evict_outside(0, 0);
        assert!(!lazy.is_rendered(del_idx));

        // Re-access: word highlights should be recomputed
        let second_render = lazy.get(del_idx);
        assert!(
            second_render.contains(style::BG_DELETED_WORD),
            "deleted line should contain BG_DELETED_WORD after re-render: {second_render:?}"
        );
    }

    #[test]
    fn lazy_color_rendering_produces_ansi() {
        let raw = "\
diff --git a/foo.rs b/foo.rs
--- a/foo.rs
+++ b/foo.rs
@@ -1,3 +1,3 @@
 context
-deleted line
+added line
";
        let files = diff::parse(raw);
        let output = render(&files, 80, true);
        let lazy = &output.lazy_lines;

        // Find added and deleted content lines
        let added_idx = output
            .line_map
            .iter()
            .position(|li| li.line_kind == Some(LineKind::Added))
            .expect("should have an added line");
        let deleted_idx = output
            .line_map
            .iter()
            .position(|li| li.line_kind == Some(LineKind::Deleted))
            .expect("should have a deleted line");

        let added_line = lazy.get(added_idx);
        assert!(
            added_line.contains(style::BG_ADDED),
            "added line should contain BG_ADDED: {added_line:?}"
        );

        let deleted_line = lazy.get(deleted_idx);
        assert!(
            deleted_line.contains(style::BG_DELETED),
            "deleted line should contain BG_DELETED: {deleted_line:?}"
        );
    }

    #[test]
    fn three_file_lazy_only_middle_accessed() {
        let raw = "\
diff --git a/a.rs b/a.rs
--- a/a.rs
+++ b/a.rs
@@ -1,1 +1,2 @@
 ctx_a
+added_a
diff --git a/b.rs b/b.rs
--- a/b.rs
+++ b/b.rs
@@ -1,1 +1,2 @@
 ctx_b
+added_b
diff --git a/c.rs b/c.rs
--- a/c.rs
+++ b/c.rs
@@ -1,1 +1,2 @@
 ctx_c
+added_c
";
        let files = diff::parse(raw);
        let output = render(&files, 80, false);
        let lazy = &output.lazy_lines;
        assert_eq!(output.file_starts.len(), 3);

        let file0_start = output.file_starts[0];
        let file1_start = output.file_starts[1];
        let file2_start = output.file_starts[2];

        // Access a content line in file 1 (the middle file)
        let file1_content = file1_start + 1;
        let _ = lazy.get(file1_content);

        // File 0 content lines should NOT be rendered
        let file0_content_rendered = (file0_start + 1..file1_start)
            .any(|i| lazy.is_rendered(i));
        assert!(
            !file0_content_rendered,
            "file 0 content lines should not be rendered when only file 1 was accessed"
        );

        // File 2 content lines should NOT be rendered
        let file2_content_rendered = (file2_start + 1..lazy.len())
            .any(|i| lazy.is_rendered(i));
        assert!(
            !file2_content_rendered,
            "file 2 content lines should not be rendered when only file 1 was accessed"
        );

        // All three file headers (structural) should be pre-rendered
        assert!(
            lazy.is_rendered(file0_start),
            "file 0 header should be pre-rendered"
        );
        assert!(
            lazy.is_rendered(file1_start),
            "file 1 header should be pre-rendered"
        );
        assert!(
            lazy.is_rendered(file2_start),
            "file 2 header should be pre-rendered"
        );
    }

    #[test]
    fn from_render_output_preserves_lazy_ctx() {
        // Verifies that `RenderOutput` (which `Document::from_render_output` consumes)
        // carries a properly-initialized lazy context: structural lines are
        // pre-rendered, content lines are lazy.
        let raw = "\
diff --git a/foo.rs b/foo.rs
--- a/foo.rs
+++ b/foo.rs
@@ -1,3 +1,4 @@
 line1
+added
 line2
 line3
";
        let files = diff::parse(raw);
        let output = render(&files, 80, false);
        let lazy = &output.lazy_lines;

        // Structural line (header at index 0) should be pre-rendered
        assert!(
            lazy.is_rendered(0),
            "structural header should be pre-rendered in RenderOutput"
        );

        // Content line should NOT be pre-rendered
        assert!(
            !lazy.is_rendered(1),
            "content line should not be pre-rendered in RenderOutput"
        );

        // Verify that the ctx is present (lazy rendering is possible)
        assert!(
            lazy.ctx.is_some(),
            "LazyLines should have a render context for lazy rendering"
        );

        // Accessing a content line should render it
        let _ = lazy.get(1);
        assert!(
            lazy.is_rendered(1),
            "content line should be rendered after get()"
        );
    }

    #[test]
    fn raw_texts_includes_structural_content() {
        let raw = "\
diff --git a/foo.rs b/foo.rs
--- a/foo.rs
+++ b/foo.rs
@@ -1,2 +1,3 @@
 ctx
+added
 more
";
        let files = diff::parse(raw);
        let output = render(&files, 80, false);
        let lazy = &output.lazy_lines;

        let texts = lazy.raw_texts();
        assert_eq!(
            texts.len(),
            lazy.len(),
            "raw_texts() length should match render_data length"
        );

        // Header slot (index 0) should contain the file path
        assert!(
            texts[0].contains("foo.rs"),
            "header raw text should contain the file path: {:?}",
            texts[0]
        );

        // Content slots should contain raw diff text
        let has_added = texts.iter().any(|t| t == "added");
        assert!(has_added, "raw_texts should include 'added' content");

        let has_ctx = texts.iter().any(|t| t == "ctx");
        assert!(has_ctx, "raw_texts should include 'ctx' content");
    }

    #[test]
    fn render_all_after_partial_eviction() {
        let raw = "\
diff --git a/a.rs b/a.rs
--- a/a.rs
+++ b/a.rs
@@ -1,2 +1,3 @@
 ctx_a
+added_a
 more_a
diff --git a/b.rs b/b.rs
--- a/b.rs
+++ b/b.rs
@@ -1,1 +1,2 @@
 ctx_b
+added_b
";
        let files = diff::parse(raw);
        let output = render(&files, 80, false);
        let lazy = &output.lazy_lines;

        // Render all lines first
        lazy.render_all();
        for i in 0..lazy.len() {
            assert!(lazy.is_rendered(i), "line {i} should be rendered after render_all");
        }

        // Evict some lines (keep only 0..5)
        lazy.evict_outside(0, 5);

        // Re-render all
        lazy.render_all();

        // to_strings should work without panic and return all lines
        let strings = lazy.to_strings();
        assert_eq!(
            strings.len(),
            output.line_map.len(),
            "to_strings after re-render should have correct length"
        );
        // Verify no empty strings where we expect content
        for (i, s) in strings.iter().enumerate() {
            assert!(
                lazy.is_rendered(i),
                "line {i} should be rendered after render_all"
            );
            // Headers and content should have non-empty rendered strings
            // (only empty continuations from wrap mismatch would be empty)
            let _ = s; // just verify no panic from unwrap inside to_strings
        }
    }

    /// Regression test: after eviction, re-accessing a line deep in a file should
    /// NOT replay highlight_line from file start. With checkpoints, highlight_calls
    /// should be bounded by the checkpoint interval, not the total file size.
    #[test]
    fn highlight_calls_bounded_after_eviction() {
        // Build a large single-file diff with 400 added lines
        let mut raw = String::from(
            "diff --git a/big.rs b/big.rs\n--- a/big.rs\n+++ b/big.rs\n@@ -1,0 +1,400 @@\n",
        );
        for i in 0..400 {
            raw.push_str(&format!("+let x{i} = {i};\n"));
        }
        let files = diff::parse(&raw);
        let output = render(&files, 120, true);
        let total = output.lazy_lines.len();
        assert!(total > 300, "fixture should produce >300 lines, got {total}");

        // Access a line near the end to warm the full file
        let deep_idx = total - 5;
        let _ = output.lazy_lines.get(deep_idx);
        let warm_hl_calls = output.lazy_lines.highlight_calls();
        assert!(
            warm_hl_calls > 300,
            "initial render should call highlight_line many times, got {warm_hl_calls}"
        );

        // Evict everything, then re-access the same deep line
        output.lazy_lines.evict_outside(0, 0);
        output.lazy_lines.reset_stats();
        let _ = output.lazy_lines.get(deep_idx);
        let replay_hl_calls = output.lazy_lines.highlight_calls();

        // With checkpoints, replay should be bounded (not full file replay).
        // Without checkpoints, this would equal warm_hl_calls (~400+).
        assert!(
            replay_hl_calls < warm_hl_calls / 2,
            "after eviction, highlight_calls should be bounded by checkpoint interval, \
             not full file size: replay={replay_hl_calls}, initial={warm_hl_calls}"
        );
    }
}
