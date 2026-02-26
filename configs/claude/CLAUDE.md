# User Preferences

## Delegation

### Behavior

**The orchestrator decides. Subagents do.**

Pass the *problem*, not the *solution* — don't read code, diagnose issues, or prescribe implementations before delegating.

**Routing reads stay inline** — status checks, branch names, file existence, small lookups that inform the next decision, and reading file lists or directory structures to scope a delegation (determining which files to pass as context). Test: "am I gathering info to choose what to do next?"

**Work gets delegated** — file analysis, diff review, artifact generation, multi-step execution. Test: "does this consume context I'll need later?"

**Evaluate work by scope, not conversational framing** — a brief continuation can be substantial work.

**Stop before investigation becomes implementation** — reading a few files to identify what to delegate is a routing read. Reading files to figure out how to implement is doing the subagent's job. Test: "am I reading to decide what to delegate, or to figure out how to implement?"

User interaction and state transitions stay in the orchestrator.

**Do NOT:**
- Read source files to analyze or diagnose problems that will be delegated
- Design implementations or prescribe code changes for subagents
- Re-read files a subagent already summarized
- Reduce a subagent to a transcriber by over-specifying the solution
- Delegate trivial routing reads that inform the orchestrator's next decision

### Routing

**Skills take precedence.** When the user's intent maps to a skill (with or without `/`), invoke it via the Skill tool — MUST NOT route directly to its subagents. Skills orchestrate subagent dispatch; the table below is what skills use internally.

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
- `TaskOutput` timeout means the result wasn't delivered in time — not that the subagent failed. MUST NOT treat a delivery timeout as evidence that delegation doesn't work; continue delegating normally
- If the same delegation fails on retry, inspect the subagent's output or error before delegating again — don't loop blindly

### Worktrees

- MUST NOT use `isolation: worktree` in agent configs or pass `isolation: "worktree"` to the Task tool. The user manages worktrees manually.

### Loops

An evaluate-fix cycle that repeats until convergence. The orchestrator drives; evaluator and fixer agents do the work.

**Structure** (repeat until converged or max iterations reached):

1. **Evaluate** — run evaluator agents in parallel; each finding MUST be `file:line — severity — one sentence`
2. **Fix** — immediately delegate all Blocking and Improvement findings to the appropriate fixer; no confirmation, no pause
3. **Re-evaluate** — run the same evaluators on updated state
4. **Converge** — stop when pass criteria are met or max iterations (default: 4) reached

**Rules:**

- MUST proceed through evaluate → fix → re-evaluate without pausing to ask the user
- **Blocking** / **Improvements** — fix immediately; escalate only when the fix has multiple plausible approaches and no available context disambiguates, or the same finding recurs after a fix attempt
- **Suggestions** — fix if quick (fewer than 3 per file); otherwise skip; do not block convergence
- When max iterations complete without convergence, present remaining findings with status and let the user decide

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

