# User Preferences

## Delegation

### Behavior

Delegate to a domain-specific subagent that owns the full problem-solving loop (read, analyze, design, write, verify, iterate). Pass the *problem*, not the *solution* — don't read code, diagnose issues, or prescribe implementations before delegating.

**Do NOT:**
- Read source files to analyze or diagnose problems that will be delegated
- Design implementations or prescribe code changes for subagents
- Re-read files a subagent already summarized
- Reduce a subagent to a transcriber by over-specifying the solution

### Routing

- Code (plan chunks) — `chunk-executor`
- Code (ad-hoc) — `code-writer`
- Commits — `committer`
- PR descriptions — `pr-writer`
- GitHub text (comments, replies, reviews) — `github-writer`
- Rules files — `rules-writer`
- Skill files — `skill-writer`
- Slide content — `slide-writer`

### Worktrees

- NEVER use `isolation: worktree` in agent configs or pass `isolation: "worktree"` to the Task tool. The user manages worktrees manually.

---

## Path Resolution

- Always resolve `tmp/` as `./tmp/` relative to the working directory, not as `/tmp/`.

## Repository Map

When the user references a repo by name (e.g., "check gadget", "look at the skill in dotfiles"), use this map to resolve the path.

**Convention:** All repos live under `~/Code/{personal,gadget,scratch}/<name>`. For repos not listed here, check those directories.

- `~/Code/personal/dotfiles` — macOS dotfiles, symlink-managed configs, **Claude Code skills**
- `~/Code/personal/slides` — Slidev presentations (pnpm workspace)
- `~/Code/gadget/gadget` — Main Gadget monorepo (app platform)
- `~/Code/gadget/ggt` — Gadget CLI tool
- `~/Code/gadget/skipper` — Kubernetes operator for Gadget apps
- `~/Code/gadget/global-infrastructure` — Terraform/infra for Gadget cloud

