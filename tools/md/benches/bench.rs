use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

use md::highlight::highlight_code;
use md::pager::find_matches;
use md::render::render_markdown;
use md::style::Style;
use md::wrap::{strip_ansi, visible_length, word_wrap};

static LARGE_README: &str = include_str!("fixtures/large-readme.md");
static CODE_HEAVY: &str = include_str!("fixtures/code-heavy.md");
static LONG_PROSE: &str = include_str!("fixtures/long-prose.md");

// ---------------------------------------------------------------------------
// render_markdown — full pipeline
// ---------------------------------------------------------------------------

fn bench_render_markdown(c: &mut Criterion) {
    let mut group = c.benchmark_group("render_markdown");

    let plain = Style::new(false, false);
    let color = Style::new(true, false);
    let pretty = Style::new(true, true);

    group.bench_function("large-readme/plain", |b| {
        b.iter(|| render_markdown(black_box(LARGE_README), 80, &plain));
    });
    group.bench_function("large-readme/color", |b| {
        b.iter(|| render_markdown(black_box(LARGE_README), 80, &color));
    });
    group.bench_function("large-readme/pretty", |b| {
        b.iter(|| render_markdown(black_box(LARGE_README), 80, &pretty));
    });

    group.bench_function("code-heavy/color", |b| {
        b.iter(|| render_markdown(black_box(CODE_HEAVY), 80, &color));
    });
    group.bench_function("code-heavy/pretty", |b| {
        b.iter(|| render_markdown(black_box(CODE_HEAVY), 80, &pretty));
    });

    group.bench_function("long-prose/color", |b| {
        b.iter(|| render_markdown(black_box(LONG_PROSE), 80, &color));
    });
    group.bench_function("long-prose/pretty", |b| {
        b.iter(|| render_markdown(black_box(LONG_PROSE), 80, &pretty));
    });

    group.finish();
}

fn bench_render_widths(c: &mut Criterion) {
    let mut group = c.benchmark_group("render_markdown/widths");
    let pretty = Style::new(true, true);

    for width in [40, 80, 120] {
        group.bench_with_input(BenchmarkId::from_parameter(width), &width, |b, &w| {
            b.iter(|| render_markdown(black_box(LONG_PROSE), w, &pretty));
        });
    }

    group.finish();
}

// ---------------------------------------------------------------------------
// word_wrap
// ---------------------------------------------------------------------------

fn bench_word_wrap(c: &mut Criterion) {
    let mut group = c.benchmark_group("word_wrap");

    let plain_paragraph = "The quick brown fox jumps over the lazy dog. \
        Pack my box with five dozen liquor jugs. \
        How vexingly quick daft zebras jump. \
        The five boxing wizards jump quickly.";

    let style = Style::new(true, true);
    let ansi_paragraph = format!(
        "The {} brown {} jumps over the {} dog. \
         Pack my {} with five dozen {} jugs. \
         How {} quick {} zebras jump.",
        style.strong_style("quick"),
        style.em_style("fox"),
        style.strong_style("lazy"),
        style.code_span("box"),
        style.em_style("liquor"),
        style.strong_style("vexingly"),
        style.em_style("daft"),
    );

    group.bench_function("plain/width-80", |b| {
        b.iter(|| word_wrap(black_box(plain_paragraph), 80, ""));
    });
    group.bench_function("plain/width-40", |b| {
        b.iter(|| word_wrap(black_box(plain_paragraph), 40, ""));
    });
    group.bench_function("ansi/width-80", |b| {
        b.iter(|| word_wrap(black_box(&ansi_paragraph), 80, ""));
    });
    group.bench_function("ansi/width-40", |b| {
        b.iter(|| word_wrap(black_box(&ansi_paragraph), 40, ""));
    });
    group.bench_function("ansi/width-40/indented", |b| {
        b.iter(|| word_wrap(black_box(&ansi_paragraph), 40, "    "));
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// strip_ansi / visible_length
// ---------------------------------------------------------------------------

fn bench_strip_ansi(c: &mut Criterion) {
    let mut group = c.benchmark_group("strip_ansi");

    let plain = "The quick brown fox jumps over the lazy dog";
    let light_ansi = "The \x1b[1mquick\x1b[22m brown \x1b[3mfox\x1b[23m jumps";
    let heavy_ansi = "\x1b[38;2;127;159;255mThe\x1b[0m \x1b[1m\x1b[38;2;255;166;87mquick\x1b[0m \
        \x1b[38;2;126;231;135mbrown\x1b[0m \x1b[3m\x1b[38;2;127;159;255mfox\x1b[0m \
        \x1b[38;2;230;237;243mjumps\x1b[0m \x1b[38;2;230;237;243mover\x1b[0m \
        \x1b[38;2;230;237;243mthe\x1b[0m \x1b[1m\x1b[38;2;255;166;87mlazy\x1b[0m \
        \x1b[38;2;230;237;243mdog\x1b[0m";

    group.bench_function("plain", |b| {
        b.iter(|| strip_ansi(black_box(plain)));
    });
    group.bench_function("light-ansi", |b| {
        b.iter(|| strip_ansi(black_box(light_ansi)));
    });
    group.bench_function("heavy-ansi", |b| {
        b.iter(|| strip_ansi(black_box(heavy_ansi)));
    });

    group.finish();
}

fn bench_visible_length(c: &mut Criterion) {
    let mut group = c.benchmark_group("visible_length");

    let plain = "The quick brown fox jumps over the lazy dog";
    let heavy_ansi = "\x1b[38;2;127;159;255mThe\x1b[0m \x1b[1m\x1b[38;2;255;166;87mquick\x1b[0m \
        \x1b[38;2;126;231;135mbrown\x1b[0m \x1b[3m\x1b[38;2;127;159;255mfox\x1b[0m \
        \x1b[38;2;230;237;243mjumps\x1b[0m";

    group.bench_function("plain", |b| {
        b.iter(|| visible_length(black_box(plain)));
    });
    group.bench_function("heavy-ansi", |b| {
        b.iter(|| visible_length(black_box(heavy_ansi)));
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// highlight_code
// ---------------------------------------------------------------------------

fn bench_highlight_code(c: &mut Criterion) {
    let mut group = c.benchmark_group("highlight_code");

    let short_js = "const x = 1;";
    let medium_rust = "\
use std::collections::HashMap;

fn main() {
    let mut map = HashMap::new();
    map.insert(\"key\", 42);
    for (k, v) in &map {
        println!(\"{k}: {v}\");
    }
}";
    let long_python = "\
import asyncio
from dataclasses import dataclass, field
from typing import Optional

@dataclass
class TreeNode:
    value: int
    left: Optional['TreeNode'] = None
    right: Optional['TreeNode'] = None
    metadata: dict = field(default_factory=dict)

    def insert(self, val: int) -> 'TreeNode':
        if val < self.value:
            if self.left is None:
                self.left = TreeNode(value=val)
            else:
                self.left.insert(val)
        elif val > self.value:
            if self.right is None:
                self.right = TreeNode(value=val)
            else:
                self.right.insert(val)
        return self

    def in_order(self) -> list[int]:
        result = []
        if self.left:
            result.extend(self.left.in_order())
        result.append(self.value)
        if self.right:
            result.extend(self.right.in_order())
        return result

async def process_tree(root: TreeNode) -> dict:
    values = root.in_order()
    await asyncio.sleep(0)
    return {'sorted': values, 'count': len(values)}";

    group.bench_function("short-js/color", |b| {
        b.iter(|| highlight_code(black_box(short_js), Some("js"), true));
    });
    group.bench_function("short-js/no-color", |b| {
        b.iter(|| highlight_code(black_box(short_js), Some("js"), false));
    });
    group.bench_function("medium-rust/color", |b| {
        b.iter(|| highlight_code(black_box(medium_rust), Some("rust"), true));
    });
    group.bench_function("long-python/color", |b| {
        b.iter(|| highlight_code(black_box(long_python), Some("python"), true));
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// find_matches (pager search)
// ---------------------------------------------------------------------------

fn bench_find_matches(c: &mut Criterion) {
    let mut group = c.benchmark_group("find_matches");

    // Simulate a 500-line rendered document
    let lines: Vec<String> = (0..500)
        .map(|i| {
            if i % 50 == 0 {
                format!("## Section {}", i / 50)
            } else if i % 10 == 0 {
                format!(
                    "The quick brown fox jumps over the lazy dog. Line {i}."
                )
            } else {
                format!(
                    "Lorem ipsum dolor sit amet, consectetur adipiscing elit. \
                     Sed do eiusmod tempor incididunt ut labore. Line {i}."
                )
            }
        })
        .collect();

    group.bench_function("common-word", |b| {
        b.iter(|| find_matches(black_box(&lines), black_box("lorem")));
    });
    group.bench_function("rare-word", |b| {
        b.iter(|| find_matches(black_box(&lines), black_box("fox")));
    });
    group.bench_function("no-match", |b| {
        b.iter(|| find_matches(black_box(&lines), black_box("zzzzzzz")));
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// Groups
// ---------------------------------------------------------------------------

criterion_group!(
    benches,
    bench_render_markdown,
    bench_render_widths,
    bench_word_wrap,
    bench_strip_ansi,
    bench_visible_length,
    bench_highlight_code,
    bench_find_matches,
);
criterion_main!(benches);
