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

- **Every phase MUST declare `**Type**:`**. Scan each `## Phase N` block for the metadata line. If any phase is missing `**Type**:` — or declares an unknown type (not one of `write`, `test`, `review`, `benchmark`) — HARD ERROR. Report which phases are unlabeled; do not proceed. Fixing this is a `plan create` re-run or a manual edit, not an execution concern.

- **`**Depends on**:` header enforcement.** If the plan header (between title and `## Brief`) has one or more `**Depends on**: <path>` lines, open each dependency plan and confirm all phases in it have every acceptance criterion checked. If any dependency is incomplete, refuse to start. Report which dependency is unmet and which criteria remain unchecked; do not proceed.

- **Identify resume point.** Find the first phase with unchecked acceptance criteria. That's where execution starts. Earlier phases are assumed complete — do not re-run them.

### 3. For each phase, in order

**a. Announce the phase.** Tell the user which phase number and title is starting, and its `**Type**:`.

**b. Load the operation instructions.** Invoke `Skill(code, <type>)`. This loads `code/operations/<type>.md` into the orchestrator's context. Follow those instructions to do the phase's work.

For `write`, `test`, and `benchmark`, the operation's instructions are the full guidance for the phase. Do the work, verify results as the operation directs.

For `review`, see step 4 — review has additional orchestrator-owned behavior beyond what `code review` alone does.

**c. Verify acceptance criteria.** For each checkbox in the phase's `### Acceptance criteria` section, confirm the criterion is met. Mark `- [x]` in the plan file. If a criterion cannot be met after reasonable effort, STOP and report to the user; do not continue to the next phase.

**d. Commit.** After all criteria pass, invoke `Skill(git, commit)` to commit the phase's changes. Each phase produces exactly one commit — the git skill decides the message per its own conventions. Do NOT amend or batch phases into one commit.

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

- **Converged** — all `**Code**:` and `**Behavior**:` criteria checked, no unresolved Blocking or Improvement findings. Mark criteria as `- [x]`, invoke `Skill(git, commit)` to commit all changes made during the loop as one review-phase commit, proceed.
- **No progress** — an iteration produces the same set of unresolved findings as the prior iteration (same files, same severities). Halt. Do NOT commit.
- **Regression** — findings increase in count or severity after a fix attempt. Halt. Do NOT commit.

**On non-convergence halt:**

- **Do NOT commit.** Any changes made during the review phase loop stay in the working tree uncommitted. The user can inspect, revert with `git restore`, or fold into a follow-up phase.
- **Append a `## Review findings` section** to the end of the plan file, listing each unresolved finding in `file:line — severity — one sentence` form, grouped under `**Code**:` and `**Behavior**:` sub-headers to match the criteria structure.
- **Prompt the user** with three choices:
  1. **Add a fixup phase to this plan.** Edit the plan to append `## Phase N+1: Fixup` with `**Type**: write` and a description derived from the findings. Re-run `plan execute`. Use when findings are narrow and within the current refactor's scope.
  2. **Spawn a new plan.** Create `tmp/<name>-fixup/plan.md` with a Brief that captures the findings as a new scope. Use when findings reveal scope creep.
  3. **Accept as-is.** Mark the findings as "acknowledged, not addressed" with rationale (inline in the `## Review findings` section). Check the criteria boxes. Invoke `Skill(git, commit)` to commit all working-tree changes as the review-phase commit. Proceed.

### 5. Skipping a phase

If a phase is genuinely unnecessary given prior phases' results, do NOT silently proceed. Instead:

- Add `**Skipped**: <rationale>` below the phase title
- Mark all its checkboxes as `- [s]`
- Inform the user which phase was skipped and why before continuing

### 6. After all phases complete

Kill any remaining dev servers or background processes. Report the full plan as done with a summary of phases completed, skipped, any `## Review findings` still open, and any cross-plan dependents that unblock now that this plan finished.

## Rules

- **Never commit during a halted review loop.** No partial commits. Working-tree changes stay uncommitted; the user decides what to keep.
- **Never amend a prior phase's commit** to retroactively fix a review finding. Create a fixup phase or a new plan instead.
- **Never auto-extend the plan mid-execution** to add phases that weren't there at validation time. The exception is the user-confirmed fixup phase on review non-convergence (step 4), which happens between executions, not during one.
- **Never bypass `**Depends on**:` enforcement.** If a dependency is incomplete, refuse. The user can remove the header if they know better, but `plan execute` does not infer.
