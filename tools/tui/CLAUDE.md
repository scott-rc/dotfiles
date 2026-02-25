# tools/tui

`tui` is a shared library crate used by `md` and `gd`. It MUST NOT contain any binary targets.

---

All public API changes MUST be backwards-compatible with both consumers (`md` and `gd`).

---

ANSI regex patterns and escape helpers live exclusively in `tui::ansi` — consumer crates MUST reuse existing constants and functions, not duplicate them.
