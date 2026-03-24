# User Preferences

## Conventions

RFC 2119 keywords (MUST, MUST NOT, SHALL, SHOULD, etc.) carry their defined meaning throughout all instructions. Treat them as binding constraints, not suggestions.

## Delegation

### Default: Work Inline

Do the work directly — read the code, make the changes, run the tests. Delegation adds overhead (context duplication, round-trip latency, loss of nuance). Only delegate when that overhead is clearly justified.

### When to Delegate

- **Scale** — task spans many files or benefits from parallel workstreams
- **Context preservation** — the work would consume context you'll need later (large diffs, extensive analysis, multi-step execution with intermediate artifacts)
- **Specialization** — a subagent handles the task type materially better

### When to Stay Inline

- Single-file edits, small fixes, config changes, typo fixes
- Investigation and implementation are tightly coupled — understanding the problem IS the fix
- The task is simple enough that delegation overhead exceeds the work itself
- You can read, change, and verify in a few steps

### When Delegating

Pass the *problem*, not the *solution* — describe what needs to change and why, not how to implement it.

**Do NOT:**
- Design implementations or prescribe code changes for subagents
- Re-read files a subagent already summarized
- Reduce a subagent to a transcriber by over-specifying the solution

### Routing

**Skills take precedence for non-trivial tasks.** Before dispatching any subagent, check whether an available skill covers the task. If a skill matches and the task is non-trivial, MUST invoke it via the Skill tool — MUST NOT bypass the skill by routing directly to its subagents. Trivial changes (single-line edits, typo fixes, config tweaks) MAY be done inline even when a skill technically matches. Skills manage their own internal routing.

---

## Timezone

- Assume the user is in **Eastern Time (ET)** — America/Toronto (Ottawa).
- When displaying or printing dates and times, MUST use ET (EST/EDT as seasonally appropriate).

## Path Resolution

- Always resolve `tmp/` as `./tmp/` relative to the working directory, not as `/tmp/`.

## Repository Map

When the user references a repo by name (e.g., "check gadget", "look at the skill in dotfiles"), use this map to resolve the path.

**Convention:** All repos live under `~/Code/{personal,gadget,scratch}/<name>`. For repos not listed here, check those directories.

- `~/Code/personal/dotfiles` — macOS dotfiles, symlink-managed configs, **Claude Code skills**
- `~/Code/gadget/gadget` — Main Gadget monorepo (app platform)
- `~/Code/gadget/ggt` — Gadget CLI tool
- `~/Code/gadget/skipper` — Kubernetes operator for Gadget apps
- `~/Code/gadget/global-infrastructure` — Terraform/infra for Gadget cloud
