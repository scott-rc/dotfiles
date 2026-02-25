---
paths:
  - "tools/**"
---

After changing a tool in `tools/`, MUST run `cargo build --release` in the tool's directory. The release binary is symlinked into PATH by `apply.sh`, so a debug-only build leaves the installed binary stale. Changes to `tools/tui/` require rebuilding both `tools/md/` and `tools/gd/`, since they depend on tui as a workspace dependency.
