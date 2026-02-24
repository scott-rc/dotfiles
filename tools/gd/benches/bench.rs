use criterion::{black_box, criterion_group, criterion_main, Criterion};

use gd::git::diff::{parse, DiffFile};
use gd::pager::tree::build_tree_entries;
use gd::render::{find_change_blocks, render, word_highlights};

static SMALL_DIFF: &str = include_str!("fixtures/small-diff.patch");
static LARGE_DIFF: &str = include_str!("fixtures/large-diff.patch");
static WORD_HEAVY_DIFF: &str = include_str!("fixtures/word-heavy-diff.patch");

fn bench_diff_parse(c: &mut Criterion) {
    let mut group = c.benchmark_group("diff_parse");
    group.bench_function("small", |b| b.iter(|| parse(black_box(SMALL_DIFF))));
    group.bench_function("large", |b| b.iter(|| parse(black_box(LARGE_DIFF))));
    group.bench_function("word-heavy", |b| {
        b.iter(|| parse(black_box(WORD_HEAVY_DIFF)))
    });
    group.finish();
}

fn bench_render(c: &mut Criterion) {
    let small_files = parse(SMALL_DIFF);
    let large_files = parse(LARGE_DIFF);
    let word_heavy_files = parse(WORD_HEAVY_DIFF);

    let mut group = c.benchmark_group("render");
    group.bench_function("small/color", |b| {
        b.iter(|| render(black_box(&small_files), 120, true))
    });
    group.bench_function("small/no-color", |b| {
        b.iter(|| render(black_box(&small_files), 120, false))
    });
    group.bench_function("large/color", |b| {
        b.iter(|| render(black_box(&large_files), 120, true))
    });
    group.bench_function("large/no-color", |b| {
        b.iter(|| render(black_box(&large_files), 120, false))
    });
    group.bench_function("word-heavy/color", |b| {
        b.iter(|| render(black_box(&word_heavy_files), 120, true))
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
        b.iter(|| word_highlights(black_box(hunk), black_box(block)))
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
        b.iter(|| build_tree_entries(black_box(&flat_files)))
    });
    group.bench_function("nested-100", |b| {
        b.iter(|| build_tree_entries(black_box(&nested_files)))
    });
    group.finish();
}

criterion_group!(
    benches,
    bench_diff_parse,
    bench_render,
    bench_word_highlights,
    bench_tree_build
);
criterion_main!(benches);
