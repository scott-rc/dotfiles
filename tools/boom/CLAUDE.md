# tools/boom

@README.md

---

Use bare unwrap where safe. See `.claude/rules/tools.md` for build and shared error-handling requirements.

---

All I/O is async (tokio). Use `tokio::task::JoinSet` for parallel operations within a tier. Use `tokio::time::timeout()` for global timeout handling.

---

Colored output goes through `output.rs` helpers (`info`, `success`, `warn`, `error`). Do not use inline ANSI codes or `colored` directly outside that module.

---

To add a new resource kind for readiness checking: add a match arm in `monitor.rs` with the kind-specific status field logic, returning `Ready`, `NotReady`, or `Failed`.
