# tools/tui

Key modules: `tui::ansi` (ANSI regex patterns and escape helpers), `tui::highlight` (syntect-based syntax highlighting, GitHub Dark theme), `tui::pager` (shared pager primitives), and `tui::search` (search state helpers).

---

`tui` is a shared library crate used by `md` and `gd`. It MUST NOT contain any binary targets.

---

All public API changes MUST be backwards-compatible with both consumers (`md` and `gd`).

---

ANSI regex patterns and escape helpers live exclusively in `tui::ansi` — consumer crates MUST reuse existing constants and functions, not duplicate them.

---

After changing tui, run `cargo test` in both `tools/md/` and `tools/gd/` to verify downstream compatibility. See `.claude/rules/tools.md` for build requirements.
