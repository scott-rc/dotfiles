# tools/md

@README.md

---

When making changes to the codebase, keep README.md in sync. Any new features, changed options, added keybindings, or modified architecture MUST be reflected in the README.

---

MUST NOT introduce custom error types, `Result` types, or error enums. Use `eprintln!()` + `process::exit(1)` for fatal errors, `unwrap_or()` for safe defaults.

---

MUST NOT duplicate ANSI regex patterns. Reuse the helpers in `tui::ansi` (re-exported via `wrap.rs`).

---

Construct styles via `Style::new(color: bool, pretty: bool)`. Palette constants live at the top of `style.rs`.

---

Follow the three fixture systems described in README.md when adding tests.

---

To add a new markdown element: add a match arm in `render_tokens()` (`render.rs`), use `Style` methods for formatting, then add a fixture `.md`/`.expected.txt` pair and register it with the appropriate macro.

---

When optimizing performance, MUST run `cargo bench` before and after changes. Use `cargo bench -- --save-baseline before` to save a baseline, then `cargo bench -- --baseline before` to compare. Use `samply record` on the release binary for flamegraph profiling.
