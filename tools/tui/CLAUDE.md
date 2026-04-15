# tools/tui

Key modules: `tui::ansi` (ANSI regex patterns and escape helpers), `tui::highlight` (syntect-based syntax highlighting, GitHub Dark theme), `tui::pager` (shared pager primitives), and `tui::search` (search state helpers).

---

`tui` is a shared library crate used by `md`. It MUST NOT contain any binary targets. (`gd` has moved to `~/Code/personal/gd/` with its own inlined copy of tui.)

---

ANSI regex patterns and escape helpers live exclusively in `tui::ansi` — consumer crates MUST reuse existing constants and functions, not duplicate them.

---

After changing tui, run `cargo test` in `tools/md/` to verify downstream compatibility. See `.claude/rules/tools.md` for build requirements.
