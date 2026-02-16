# tools/md

Terminal markdown renderer CLI. Built by parent `apply.sh`, installed to `~/.cargo/bin/md`. Run `cargo test` from this directory.

---

No custom error types. Use `eprintln!()` + `process::exit(1)` for fatal errors, `unwrap_or()` for safe defaults. Do not introduce `Result` types or error enums.

---

All ANSI escape handling lives in `wrap.rs`: `strip_ansi()`, `visible_length()`, `split_ansi()`. Reuse these — do not duplicate ANSI regex patterns.

---

Single `Style` struct constructed via `Style::new(color: bool)`. All formatting methods return plain text when `color = false`. Palette constants are at the top of `style.rs`.

---

Three fixture systems:

- Rendering: `.md` + `.expected.txt` pairs in `fixtures/rendering/`, registered via `rendering_fixture!` / `frontmatter_fixture!` macros in `render.rs` tests. Width 60, no color.
- JSON: Per-module fixtures in `fixtures/{module}/`, loaded via `include_str!()` + custom `Deserialize` structs.
- Integration: `tests/integration.rs` spawns binary via `CARGO_BIN_EXE_md`, uses `run_md()` helper.

---

To add a new markdown element: add a match arm in `render_tokens()` (`render.rs`), use `Style` methods for formatting, then add a fixture `.md`/`.expected.txt` pair and register it with the appropriate macro.
