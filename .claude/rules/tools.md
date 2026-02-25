---
paths:
  - "tools/**"
---

- After changing any file under `tools/`, run `cargo build --release` from `tools/` (the workspace root). The release binary is symlinked into PATH by `apply.sh`; a debug-only build leaves the installed binary stale.
- Changes to `tools/tui/` require rebuilding both `tools/md/` and `tools/gd/`, since both depend on tui as a workspace path dependency.
- MUST NOT introduce custom error types, `Result` types, or error enums. Use `eprintln!()` + `process::exit(1)` for fatal errors.
