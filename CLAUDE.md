# CLAUDE.md

@README.md

---

When making changes to the codebase, keep README.md in sync. Any new features, changed options, added keybindings, or modified architecture MUST be reflected in the README.

---

After changing a tool in `tools/`, MUST run `cargo build --release` in the tool's directory. The release binary is symlinked into PATH by `apply.sh`, so a debug-only build leaves the installed binary stale.