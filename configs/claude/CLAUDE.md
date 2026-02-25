# User Preferences

## Delegation

For code, commits, PR descriptions, GitHub text, rules files, skill files, and slide content — delegate to a domain-specific subagent that owns the write-verify-retry loop rather than writing directly. The main agent researches, loads context, and picks the right subagent; subagents write, verify, and iterate on failures.

Route by task:
- Code (plan chunks) — `chunk-executor`
- Code (ad-hoc) — `code-writer`
- Commits — `committer`
- PR descriptions — `pr-writer`
- GitHub text (comments, replies, reviews) — `github-writer`
- Rules files — `rules-writer`
- Skill files — `skill-writer`
- Slide content — `slide-writer`

---

## Path Resolution

- Always resolve `tmp/` as `./tmp/` relative to the working directory, not as `/tmp/`.

## Repository Map

When the user references a repo by name (e.g., "check gadget", "look at the skill in dotfiles"), use this map to resolve the path.

**Convention:** All repos live under `~/Code/{personal,gadget,scratch}/<name>`. For repos not listed here, check those directories.

### Personal

- `~/Code/personal/dotfiles` — macOS dotfiles, symlink-managed configs, **Claude Code skills**
- `~/Code/personal/recipe-book` — Recipe collection
- `~/Code/personal/slides` — Slidev presentations (pnpm workspace)

### Gadget (work)

- `~/Code/gadget/gadget` — Main Gadget monorepo (app platform)
- `~/Code/gadget/ggt` — Gadget CLI tool
- `~/Code/gadget/skipper` — Kubernetes operator for Gadget apps
- `~/Code/gadget/global-infrastructure` — Terraform/infra for Gadget cloud
- `~/Code/gadget/js-clients` — JavaScript client libraries

