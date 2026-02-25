# tools/md

@README.md

---

Use `unwrap_or()` for safe defaults. See `.claude/rules/tools.md` for build and shared error-handling requirements.

---

Construct styles via `Style::new(color: bool, pretty: bool)`. Palette constants live at the top of `style.rs`.

---

To add a new markdown element: add a match arm in `render_tokens()` (`render.rs`), use `Style` methods for formatting, then add a fixture `.md`/`.expected.txt` pair and register it with the appropriate macro.

---

MUST run benchmarks before and after performance-sensitive changes:
- `cargo bench --bench bench -- --save-baseline before` to save a baseline
- make changes
- `cargo bench --bench bench -- --baseline before` to compare
- `samply record ./target/release/md --no-pager ../../README.md` for flamegraph profiling
