# tools/tui

`tui` is a shared library crate used by `md` and `gd`. It MUST NOT contain any binary targets.

---

All public API changes MUST be backwards-compatible with both consumers (`md` and `gd`).

---

ANSI regex patterns live exclusively in `tui::ansi` and MUST NOT be duplicated in consumer crates.

---

When making changes to the codebase, keep README.md in sync. Any new features, changed options, added keybindings, or modified architecture MUST be reflected in the README.
