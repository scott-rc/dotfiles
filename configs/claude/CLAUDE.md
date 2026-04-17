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

## Ask When Unsure

Default to action when the path is obvious. Ask when the answer genuinely changes what gets built or how — ambiguous requirements with meaningfully different scope, multiple valid approaches with real trade-offs, uncertain side effects, or missing context that can't be safely inferred. When you do ask, lead with a recommendation, be concrete about the options, and batch related unknowns into a single question.

### When Not to Ask

- Trivial style choices with a clear convention in the codebase
- Decisions the user can easily reverse
- Anything Claude can verify or infer from the existing code without guessing
- Plan iteration — when the user provides feedback on a plan, incorporate it and present the updated plan; do not ask permission to incorporate feedback the user just gave
- Obvious next step — if the user pointed out a problem or gap, address it directly; do not ask "should I fix this?"

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
- `~/Code/personal/gd` — Terminal git diff viewer (Rust), standalone repo
- `~/Code/gadget/gadget` — Main Gadget monorepo
- `~/Code/gadget/ggt` — Gadget CLI tool
- `~/Code/gadget/skipper` — Kubernetes operator for Gadget apps
- `~/Code/gadget/global-infrastructure` — Terraform/infra for Gadget cloud
