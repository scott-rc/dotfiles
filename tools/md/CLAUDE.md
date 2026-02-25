# tools/md

@README.md

---

MUST NOT introduce custom error types, `Result` types, or error enums. Use `eprintln!()` + `process::exit(1)` for fatal errors, `unwrap_or()` for safe defaults.

---

Construct styles via `Style::new(color: bool, pretty: bool)`. Palette constants live at the top of `style.rs`.

---

To add a new markdown element: add a match arm in `render_tokens()` (`render.rs`), use `Style` methods for formatting, then add a fixture `.md`/`.expected.txt` pair and register it with the appropriate macro.

---

MUST run benchmarks before and after performance-sensitive changes.
