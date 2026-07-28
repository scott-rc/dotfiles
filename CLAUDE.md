# CLAUDE.md

## Architecture Summary

### Setup

- `apply.sh` is the main setup script: installs Homebrew/Nix, runs `brew bundle` on `Brewfile`, creates symlinks, builds Cargo tools, installs md2pdf's pnpm dependencies
- Symlinks use `ensure_symlink()` and back up existing files to `.bak`

### Cargo Tools

- `tools/tui/` is a workspace dependency of `tools/md/`
- `gd` lives in its own repo at `~/Code/personal/gd/` (inlines its own copy of `tui`)

### Node Tools

- `tools/md2pdf/` -- markdown-to-PDF CLI: react-markdown + Tailwind v4 typography compiled at render time, Inter embedded as data URIs, Puppeteer print to A4; pnpm-managed, runs from source via tsx (no build step); exposed globally via the `md2pdf` fish function

### Fish Shell

- `configs/fish/conf.d/` -- auto-loaded config files; each MUST have `status is-interactive` guard

## Claude Code

- Skills: `configs/claude/skills/`
- Rules: `configs/claude/rules/`

## Build & Test

- `./apply.sh` -- full setup (Homebrew, Nix, symlinks, Cargo tools)
- `cargo build --release` in `tools/` (workspace root) -- build all Cargo workspace tools
- `cargo test` in `tools/<name>/` -- per-Cargo-tool tests
- `pnpm typecheck` / `pnpm test` in `tools/md2pdf/` -- md2pdf TypeScript check and Vitest suite

---

When making changes to the codebase, keep README.md in sync. Any new features, changed options, added keybindings, or modified architecture MUST be reflected in the README.
