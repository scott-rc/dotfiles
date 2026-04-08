use criterion::{black_box, criterion_group, criterion_main, Criterion};

use gd::git::diff::{parse, DiffFile};
use gd::pager::{bench_layout, bench_render_frame, bench_style_files, tree::build_tree_entries};
use gd::render::{apply_diff_colors, find_change_blocks, render, word_highlights};
use gd::style::{BG_ADDED, BG_ADDED_WORD};

static SMALL_DIFF: &str = include_str!("fixtures/small-diff.patch");
static LARGE_DIFF: &str = include_str!("fixtures/large-diff.patch");
static WORD_HEAVY_DIFF: &str = include_str!("fixtures/word-heavy-diff.patch");

fn bench_diff_parse(c: &mut Criterion) {
    let mut group = c.benchmark_group("diff_parse");
    group.bench_function("small", |b| b.iter(|| parse(black_box(SMALL_DIFF))));
    group.bench_function("large", |b| b.iter(|| parse(black_box(LARGE_DIFF))));
    group.bench_function("word-heavy", |b| {
        b.iter(|| parse(black_box(WORD_HEAVY_DIFF)));
    });
    group.finish();
}

fn bench_render(c: &mut Criterion) {
    let small_files = parse(SMALL_DIFF);
    let large_files = parse(LARGE_DIFF);
    let word_heavy_files = parse(WORD_HEAVY_DIFF);

    let mut group = c.benchmark_group("render");
    group.bench_function("small/color", |b| {
        b.iter(|| render(black_box(&small_files), 120, true));
    });
    group.bench_function("small/no-color", |b| {
        b.iter(|| render(black_box(&small_files), 120, false));
    });
    group.bench_function("large/color", |b| {
        b.iter(|| render(black_box(&large_files), 120, true));
    });
    group.bench_function("large/no-color", |b| {
        b.iter(|| render(black_box(&large_files), 120, false));
    });
    group.bench_function("word-heavy/color", |b| {
        b.iter(|| render(black_box(&word_heavy_files), 120, true));
    });
    group.finish();
}

fn bench_word_highlights(c: &mut Criterion) {
    let files = parse(WORD_HEAVY_DIFF);
    let hunk = &files[0].hunks[0];
    let blocks = find_change_blocks(hunk);
    let block = blocks
        .iter()
        .find(|b| !b.deleted.is_empty() && !b.added.is_empty())
        .expect("word-heavy fixture should have a block with both deleted and added lines");

    let mut group = c.benchmark_group("word_highlights");
    group.bench_function("word-heavy/first-block", |b| {
        b.iter(|| word_highlights(black_box(hunk), black_box(block)));
    });
    group.finish();
}

fn bench_apply_diff_colors(c: &mut Criterion) {
    let files = parse(WORD_HEAVY_DIFF);
    let rendered = render(&files, 120, true);

    let hunk = &files[0].hunks[0];
    let blocks = find_change_blocks(hunk);
    let block = blocks
        .iter()
        .find(|b| !b.deleted.is_empty() && !b.added.is_empty())
        .expect("word-heavy fixture should have a block with both deleted and added lines");

    let (_, add_hl) = word_highlights(hunk, block);
    let word_ranges = &add_hl[0];

    let first_added_idx = block.added[0];
    let raw = hunk.lines[first_added_idx].content.as_str();

    let all_lines = rendered.lines();
    let syntax_colored = all_lines
        .iter()
        .find(|l| l.contains(BG_ADDED))
        .or_else(|| all_lines.iter().find(|l| !l.is_empty()))
        .map_or("", String::as_str);

    let mut group = c.benchmark_group("apply_diff_colors");
    group.bench_function("word-heavy/added-line", |b| {
        b.iter(|| {
            apply_diff_colors(
                black_box(syntax_colored),
                black_box(raw),
                black_box(BG_ADDED),
                black_box(BG_ADDED_WORD),
                black_box(word_ranges),
                black_box(true),
            );
        });
    });
    group.bench_function("word-heavy/no-ranges", |b| {
        b.iter(|| {
            apply_diff_colors(
                black_box(syntax_colored),
                black_box(raw),
                black_box(BG_ADDED),
                black_box(BG_ADDED_WORD),
                black_box(&[] as &[(usize, usize)]),
                black_box(true),
            );
        });
    });
    group.finish();
}

fn bench_tree_build(c: &mut Criterion) {
    let flat_files: Vec<DiffFile> = (1..=10)
        .map(|i| DiffFile::from_content(&format!("file_{i:02}.rs"), "line\n"))
        .collect();

    let dirs = [
        "src/components/auth",
        "src/components/dashboard",
        "src/lib/utils",
        "src/lib/hooks",
        "src/api/routes",
        "src/api/middleware",
        "src/models",
        "src/config",
        "tests/unit",
        "tests/integration",
    ];
    let mut nested_files: Vec<DiffFile> = (0..100)
        .map(|i| {
            let dir = dirs[i % dirs.len()];
            DiffFile::from_content(&format!("{dir}/file_{i:03}.rs"), "line\n")
        })
        .collect();
    nested_files.sort_by(|a, b| a.path().cmp(b.path()));

    let mut group = c.benchmark_group("tree_build");
    group.bench_function("flat-10", |b| {
        b.iter(|| build_tree_entries(black_box(&flat_files)));
    });
    group.bench_function("nested-100", |b| {
        b.iter(|| build_tree_entries(black_box(&nested_files)));
    });
    group.finish();
}

fn bench_style_files_fn(c: &mut Criterion) {
    let large_files = parse(LARGE_DIFF);
    let word_heavy_files = parse(WORD_HEAVY_DIFF);

    let mut group = c.benchmark_group("style_files");
    group.bench_function("large/color", |b| {
        b.iter(|| bench_style_files(black_box(&large_files), true));
    });
    group.bench_function("word-heavy/color", |b| {
        b.iter(|| bench_style_files(black_box(&word_heavy_files), true));
    });
    group.finish();
}

fn bench_layout_fn(c: &mut Criterion) {
    let large_files = parse(LARGE_DIFF);
    let word_heavy_files = parse(WORD_HEAVY_DIFF);

    let large_styled = bench_style_files(&large_files, true);
    let word_heavy_styled = bench_style_files(&word_heavy_files, true);

    let mut group = c.benchmark_group("layout");
    group.bench_function("large/120", |b| {
        b.iter(|| bench_layout(black_box(&large_styled), 120, true));
    });
    group.bench_function("word-heavy/120", |b| {
        b.iter(|| bench_layout(black_box(&word_heavy_styled), 120, true));
    });
    // Relayout at different width (simulates tree toggle)
    group.bench_function("large/80", |b| {
        b.iter(|| bench_layout(black_box(&large_styled), 80, true));
    });
    group.finish();
}

fn bench_render_content_area(c: &mut Criterion) {
    let files = parse(WORD_HEAVY_DIFF);
    let cols: u16 = 120;
    let rows: u16 = 50;

    let mut group = c.benchmark_group("render_content_area");
    group.bench_function("frame", |b| {
        b.iter_batched(
            || render(&files, cols as usize, true),
            |output| {
                let mut sink = Vec::with_capacity(16 * 1024);
                bench_render_frame(output, black_box(&files), &mut sink, cols, rows);
                black_box(sink);
            },
            criterion::BatchSize::SmallInput,
        );
    });
    group.finish();
}

criterion_group!(
    benches,
    bench_diff_parse,
    bench_render,
    bench_word_highlights,
    bench_apply_diff_colors,
    bench_tree_build,
    bench_style_files_fn,
    bench_layout_fn,
    bench_render_content_area
);
criterion_main!(benches);
