# tools/zellij-sync-stacks

@README.md

---

When making changes to the codebase, keep README.md in sync. Any new features, changed options, added keybindings, or modified architecture MUST be reflected in the README.

---

All pure logic (types and functions) MUST be platform-independent — no `#[cfg(target_arch = "wasm32")]` gate, no `zellij_tile` imports. Only the plugin state, `ZellijPlugin` impl, and `register_plugin!` are WASM-gated.

---

Keep it single-file (`main.rs`). Do not split into modules unless the file exceeds ~1000 lines.

---

MUST NOT introduce custom error types or `Result` types. Use `eprintln!()` for debug logging and `Option` returns for fallible operations.
