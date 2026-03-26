# User Preferences

## Delegation

### Default: Work Inline

Do the work directly — read the code, make the changes, run the tests. Delegation adds overhead (context duplication, round-trip latency, loss of nuance). Only delegate when that overhead is clearly justified.

### When to Delegate

- **Scale** — task spans many files or benefits from parallel workstreams
- **Context preservation** — the work would consume context you'll need later (large diffs, extensive analysis, multi-step execution with intermediate artifacts)
- **Specialization** — a subagent handles the task type materially better

### When Delegating

Pass the _problem_, not the _solution_ — describe what needs to change and why, not how to implement it.

**Do NOT:**

- Design implementations or prescribe code changes for subagents
- Re-read files a subagent already summarized
- Reduce a subagent to a transcriber by over-specifying the solution

### Routing

If a skill covers the task and it's non-trivial, MUST invoke it via the Skill tool — MUST NOT bypass skills by routing directly to their subagents. Trivial changes MAY be done inline.

---

## Timezone

- Assume the user is in **Eastern Time (ET)** — America/Toronto.
- When displaying or printing dates and times, MUST use ET (EST/EDT as seasonally appropriate).

## Path Resolution

- Always resolve `tmp/` as `./tmp/` relative to the working directory, not as `/tmp/`.

## Repository Map

When the user references a repo by name (e.g., "check gadget", "look at the skill in dotfiles"), use this map to resolve the path.

**Convention:** All repos live under `~/Code/{personal,gadget,scratch}/<name>`. For repos not listed here, check those directories.

- `~/Code/personal/dotfiles` — macOS dotfiles, symlink-managed configs, **Claude Code skills**
- `~/Code/gadget/gadget` — Main Gadget monorepo
- `~/Code/gadget/ggt` — Gadget CLI tool
- `~/Code/gadget/skipper` — Kubernetes operator for Gadget apps
- `~/Code/gadget/global-infrastructure` — Terraform/infra for Gadget cloud
