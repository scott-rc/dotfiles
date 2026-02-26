# User Preferences

## Delegation

### Behavior

**The orchestrator decides. Subagents do.**

Pass the *problem*, not the *solution* — don't read code, diagnose issues, or prescribe implementations before delegating.

**Routing reads stay inline** — status checks, branch names, file existence, small lookups that inform the next decision. Test: "am I gathering info to choose what to do next?"

**Work gets delegated** — file analysis, diff review, artifact generation, multi-step execution. Test: "does this consume context I'll need later?"

User interaction and state transitions stay in the orchestrator.

**Do NOT:**
- Read source files to analyze or diagnose problems that will be delegated
- Design implementations or prescribe code changes for subagents
- Re-read files a subagent already summarized
- Reduce a subagent to a transcriber by over-specifying the solution
- Delegate trivial routing reads that inform the orchestrator's next decision

### Routing

- Code (plan chunks) — `chunk-executor`
- Code (ad-hoc) — `code-writer`
- Commits — `committer`
- PR descriptions — `pr-writer`
- GitHub text (comments, replies, reviews) — `github-writer`
- Rules files — `rules-writer`
- Skill files — `skill-writer`
- Slide content — `slide-writer`

### Background Agents

- Run subagents in **foreground** (default) when you need results before continuing
- If you use `run_in_background: true`, use `TaskOutput` with `block: true` to wait — do NOT attempt to `resume` a running agent (it errors with "Cannot resume agent: it is still running")

### Worktrees

- MUST NOT use `isolation: worktree` in agent configs or pass `isolation: "worktree"` to the Task tool. The user manages worktrees manually.

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

