# tools/gd

@README.md

---

When making changes to the codebase, keep README.md in sync. Any new features, changed options, added keybindings, or modified architecture MUST be reflected in the README.

---

Use `nvim.call()` for RPC calls not in nvim-rs typed API (e.g. `nvim_ui_attach`).

---

Only render ratatui frame after nvim sends `Flush` redraw event.

---

MUST NOT introduce custom error types, `Result` types, or error enums. Use `eprintln!()` + `process::exit(1)` for fatal errors, bare unwrap where safe.

---

The nvim-rs Writer type is `Compat<ChildStdin>` (from `tokio_util::compat`), aliased as `bridge::Writer`. Use this alias everywhere, not raw `ChildStdin`.
