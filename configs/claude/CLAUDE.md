# User Preferences

## Delegation

### Behavior

**The orchestrator decides. Subagents do.**

Pass the *problem*, not the *solution* — don't read code, diagnose issues, or prescribe implementations before delegating.

**Routing reads stay inline** — status checks, branch names, file existence, small lookups that inform the next decision, and reading file lists or directory structures to scope a delegation (determining which files to pass as context). Test: "am I gathering info to choose what to do next?"

**Work gets delegated** — file analysis, diff review, artifact generation, multi-step execution. Test: "does this consume context I'll need later?"

**Stop before investigation becomes implementation** — reading a few files to identify what to delegate is a routing read. Reading files to figure out how to implement is doing the subagent's job.

User interaction and state transitions stay in the orchestrator.

**Do NOT:**
- Read source files to analyze or diagnose problems that will be delegated (reading file lists to scope is fine — see routing reads above)
- Design implementations or prescribe code changes for subagents
- Re-read files a subagent already summarized
- Reduce a subagent to a transcriber by over-specifying the solution
- Delegate trivial routing reads that inform the orchestrator's next decision

### Routing

**Skills take precedence.** When the user's intent maps to a skill (with or without `/`), invoke it via the Skill tool — MUST NOT route directly to its subagents. The table below covers direct subagent dispatch when no skill applies. Skills manage their own internal routing.

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

- `~/Code/personal/dotfiles` — macOS dotfiles, symlink-managed configs, **Claude Code skills**
- `~/Code/personal/slides` — Slidev presentations (pnpm workspace)
- `~/Code/gadget/gadget` — Main Gadget monorepo (app platform)
- `~/Code/gadget/ggt` — Gadget CLI tool
- `~/Code/gadget/skipper` — Kubernetes operator for Gadget apps
- `~/Code/gadget/global-infrastructure` — Terraform/infra for Gadget cloud
