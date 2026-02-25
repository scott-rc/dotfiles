# tools/gd

@README.md

---

Use bare unwrap where safe. See `.claude/rules/tools.md` for build and shared error-handling requirements.

---

Diff color constants live at the top of `style.rs`. Use `style::` prefix everywhere, not inline ANSI codes.

---

When optimizing performance, MUST run `cargo bench` before and after changes. Use `cargo bench --bench bench -- --save-baseline before` to save a baseline, then `cargo bench --bench bench -- --baseline before` to compare. Use `samply record` on the release binary for flamegraph profiling.
