use std::time::{Duration, Instant};

use gd::git::diff::parse;
use gd::render::render;

static LARGE_DIFF: &str = include_str!("../benches/fixtures/large-diff.patch");

#[test]
#[ignore] // run with --release --ignored; too slow in debug, flaky on loaded CI
fn render_large_color_under_threshold() {
    let files = parse(LARGE_DIFF);
    let iterations = 10;

    let start = Instant::now();
    for _ in 0..iterations {
        let _ = render(&files, 120, true);
    }
    let avg = start.elapsed() / iterations;

    assert!(
        avg < Duration::from_millis(45),
        "render/large/color averaged {avg:?}, threshold is 45ms"
    );
}
