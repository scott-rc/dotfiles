# Execute Plan

Orchestrate phase-by-phase plan execution with commit checkpoints, cross-plan dependency enforcement, and a loop-based terminal review phase.

## Model

`plan execute` runs in the top-level conversation. It is the orchestrator. For each phase, it invokes `Skill(code, <type>)` — which is **instruction loading, not subagent dispatch**. The orchestrator loads the named operation's instructions into its own context and follows them itself. User interactions (clarification questions, design picks, etc.) happen naturally in the orchestrator.

There are no subagents in the phase-execution path. Subagents appear only inside specific operations when they explicitly parallelize bounded work (e.g. `code review`'s per-file decomposition), and those return before any user decision.

## Instructions

### 1. Load the plan file

If the user provided a path, use it. Otherwise search for plan files in `./tmp/*/plan.md`. If exactly one is found, use it. If multiple are found, present them as options. If none are found, ask the user for the path.

### 2. Validate plan structure

Before running any phase:

- **Every phase MUST declare `**Type**:`**. Scan each `## Phase N` block for the metadata line. If any phase is missing `**Type**:` — or declares an unknown type (not one of `write`, `test`, `review`, `benchmark`, `audit`) — HARD ERROR. Report which phases are unlabeled; do not proceed. Fixing this is a `plan create` re-run or a manual edit, not an execution concern.

- **`**Depends on**:` header enforcement.** If the plan header (between title and `## Brief`) has one or more `**Depends on**: <path>` lines, open each dependency plan and confirm all phases in it have every acceptance criterion checked. If any dependency is incomplete, refuse to start. Report which dependency is unmet and which criteria remain unchecked; do not proceed.

- **Identify resume point.** Find the first phase with unchecked acceptance criteria. That's where execution starts. Earlier phases are assumed complete — do not re-run them.

### 3. For each phase, in order

**a. Announce the phase.** Tell the user which phase number and title is starting, and its `**Type**:`.

**b. Load the operation instructions.** For `write`, `test`, and `benchmark`, invoke `Skill(code, <type>)`. This loads `code/operations/<type>.md` into the orchestrator's context. Follow those instructions to do the phase's work.

For `review`, see step 4 — review has additional orchestrator-owned behavior beyond what `code review` alone does.

For `audit`, there is no code-skill dispatch. Audit is orchestrator-owned. Consult `plan/references/phase-templates.md` under `## Type: audit` for the generic protocol, and follow the phase's own `### What to build` for the audit surface and category set. Audit is currently a STUB Type — its spec is deliberately minimal, and learnings from each real use should be captured back into `phase-templates.md`.

**c. Verify acceptance criteria.** For each checkbox in the phase's `### Acceptance criteria` section, confirm the criterion is met. Mark `- [x]` in the plan file. If a criterion cannot be met after reasonable effort, STOP and report to the user; do not continue to the next phase.

**d. Commit.** After all criteria pass, invoke `Skill(git, commit)` to commit the phase's changes. Each phase produces exactly one commit — the git skill decides the message per its own conventions. Do NOT amend or batch phases into one commit.

After the commit returns, capture the resulting SHA (`git rev-parse HEAD`) and record it on a `**Commit**:` line directly under the phase title, alongside the existing `**Type**:` line:

```markdown
## Phase N: <Title>

**Type**: <type>
**Commit**: <full-sha>

### What to build
...
```

This mapping lets `plan review` cross-reference every phase against its commit and detect Scope-creep commits — commits in the Base-SHA-to-HEAD range that aren't tied to any phase.

**e. Clean up.** Kill any dev servers or background processes started during this phase that aren't needed later.

### 4. Review phase behavior

A phase with `**Type**: review` runs as an evaluate-fix loop combining static and behavioral verification. It is the only phase type where the orchestrator does meaningful work beyond loading one operation's instructions.

**Loop structure:**

1. **Evaluate (static).** Invoke `Skill(code, review)`. Follow the review operation's instructions against the phase's `**Code**:` criteria. Collect findings.

2. **Evaluate (behavioral).** For each `**Behavior**:` criterion in the phase, exercise the running artifact directly: start a dev server if needed, drive the TUI (via replay mode or manual use), take screenshots with preview tools, or ask the user to confirm what can't be automated. Collect findings that fail criteria.

3. **Fix.** Address every Blocking and Improvement finding from either evaluator. Static fixes happen per `code review`'s own loop mode (which runs Skill(code, write) internally when findings imply code changes). Behavioral fixes invoke `Skill(code, write)` directly in TDD mode (regression sub-case) — write a failing test that reproduces the bug, then fix it.

4. **Re-evaluate.** Re-run both static and behavioral evaluation against updated state.

5. **Check termination.** Continue the loop or stop per the conditions below.

**Termination:**

- **Converged** — all `**Code**:` and `**Behavior**:` criteria checked, no unresolved Blocking or Improvement findings. Mark criteria as `- [x]`, invoke `Skill(git, commit)` to commit all changes made during the loop as one review-phase commit, record the SHA on the review phase's `**Commit**:` line, proceed to step 6.
- **No progress** — an iteration produces the same set of unresolved findings as the prior iteration (same files, same severities). Halt. Do NOT commit.
- **Regression** — findings increase in count or severity after a fix attempt. Halt. Do NOT commit.

**On non-convergence halt:**

- **Do NOT commit.** Any changes made during the review phase loop stay in the working tree uncommitted. The user can inspect, revert with `git restore`, or fold into a follow-up phase.
- **Append a Halt-findings section** (`## Review findings`) to the end of the plan file, listing each unresolved finding in `file:line — severity — one sentence` form, grouped under `**Code**:` and `**Behavior**:` sub-headers to match the criteria structure.
- **Hand off to `plan review`.** Do NOT prompt the user inline for fixup vs new-plan vs acknowledge — that decision flow lives in `plan review` now. Proceed to step 6, which invokes `Skill(plan, review)` regardless of outcome.

### 5. Skipping a phase

If a phase is genuinely unnecessary given prior phases' results, do NOT silently proceed. Instead:

- Add `**Skipped**: <rationale>` below the phase title
- Mark all its checkboxes as `- [s]`
- Inform the user which phase was skipped and why before continuing

### 6. Hand off to `plan review`

Regardless of outcome — clean completion of all phases, halt on non-convergence, full skip, or partial skip — invoke `Skill(plan, review)` on the current plan file as the final step of execution. `plan review` owns the retrospective, the three-way resolution decision for any Open items (Fixup phase / New plan / Acknowledgment), and the Auto-continue prompt when Fixup phases are appended.

Before handing off: kill any remaining dev servers or background processes started during this plan that aren't needed later.

Do NOT report completion yourself — `plan review` produces the final report, which includes the Retrospective section path, the count of Fixup phases appended, any New plans spawned, and any cross-plan dependents that now unblock.

## Rules

- **Never commit during a halted review loop.** No partial commits. Working-tree changes stay uncommitted; `plan review` surfaces them as an Open item.
- **Never amend a prior phase's commit** to retroactively fix a review finding. `plan review` proposes a Fixup phase or a New plan instead.
- **Never auto-extend the plan mid-execution** to add phases that weren't there at validation time. New phases only land via `plan review`'s confirmed Fixup-phase flow, which happens between executions — not during one.
- **Never bypass `**Depends on**:` enforcement.** If a dependency is incomplete, refuse. The user can remove the header if they know better, but `plan execute` does not infer.
- **Never skip the handoff to `plan review`.** Every `plan execute` ends with `plan review`, even on clean completion with zero open items. On a clean run, the Retrospective is a one-line summary and exits immediately.
