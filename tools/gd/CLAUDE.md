# tools/gd

@README.md

---

Use bare unwrap where safe. See `.claude/rules/tools.md` for build and shared error-handling requirements.

---

Diff color constants live at the top of `style.rs`. Use `style::` prefix everywhere, not inline ANSI codes.

---

Fix clippy lints idiomatically -- narrow `pub` to `pub(crate)` for crate-internal items, refactor to reduce argument counts, use `let...else` and `is_none_or`. MUST NOT suppress with `#[allow]` unless the lint is genuinely inapplicable.

---

When optimizing performance, MUST run `cargo bench` before and after changes. Use `cargo bench --bench bench -- --save-baseline before` to save a baseline, then `cargo bench --bench bench -- --baseline before` to compare. Use `samply record` on the release binary for flamegraph profiling.

For end-to-end startup benchmarking, use `--replay q` (NOT `--no-pager` -- it disables color when piped, hiding the real bottleneck). Use `GD_DEBUG=1` to get phase-level timing: `GD_DEBUG=1 gd --replay q 2>timing.txt`.
