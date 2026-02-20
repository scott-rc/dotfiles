# tools/gd

@README.md

---

When making changes to the codebase, keep README.md in sync. Any new features, changed options, added keybindings, or modified architecture MUST be reflected in the README.

---

MUST NOT introduce custom error types, `Result` types, or error enums. Use `eprintln!()` + `process::exit(1)` for fatal errors, bare unwrap where safe.

---

MUST NOT duplicate ANSI regex patterns. Reuse the helpers in `ansi.rs`.

---

Diff color constants live at the top of `style.rs`. Use `style::` prefix everywhere, not inline ANSI codes.
