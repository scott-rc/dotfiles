# User Preferences

## Path Resolution

- Always resolve `tmp/` as `./tmp/` relative to the working directory, not as `/tmp/`.

## Repository Map

When the user references a repo by name (e.g., "check gadget", "look at the skill in dotfiles"), use this map to resolve the path.

**Convention:** All repos live under `~/Code/{personal,gadget,scratch}/<name>`. For repos not listed here, check those directories.

### Personal

- `~/Code/personal/dotfiles` — macOS dotfiles, symlink-managed configs, **Claude Code skills** (see Skills below)
- `~/Code/personal/recipe-book` — Recipe collection
- `~/Code/personal/slides` — Slidev presentations (pnpm workspace)

### Gadget (work)

- `~/Code/gadget/gadget` — Main Gadget monorepo (app platform)
- `~/Code/gadget/ggt` — Gadget CLI tool
- `~/Code/gadget/skipper` — Kubernetes operator for Gadget apps
- `~/Code/gadget/global-infrastructure` — Terraform/infra for Gadget cloud
- `~/Code/gadget/js-clients` — JavaScript client libraries

### Skills Locations

- **Global skills** (available everywhere): `~/Code/personal/dotfiles/configs/claude/skills/`
- **Gadget-specific skills** (only available in gadget projects): `~/Code/gadget/gadget/.claude/skills/`
