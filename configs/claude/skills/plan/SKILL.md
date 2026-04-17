---
name: plan
description: Turn a Brief-populated plan file into phased work, then execute it phase by phase with commit checkpoints. Use when the user wants to break down a PRD or refactor Brief into phases, create an implementation plan, execute a plan, continue implementing, or mentions "tracer bullets" or "vertical slices".
argument-hint: "[create | execute] [path]"
---

# Plan

Create and execute phased implementation plans. Plans live as single files at `tmp/<name>/plan.md` — one file per piece of work, from initial Brief through final review. Plans are seeded by `prd` (for features) or `code architect <target>` (for refactors); each seeder writes the Brief and hands off to `plan create`.

## Operations

### Create
Read a Brief-populated plan file and append phases to it. Assigns `**Type**:` per phase (`write`, `test`, `review`, `benchmark`); pulls per-type starter acceptance criteria from `references/phase-templates.md`; appends a default terminal review phase whose criteria derive from the Brief's `### Review Criteria` section.
MUST read operations/create.md before executing.

### Execute
Run the plan phase by phase. Validates that every phase has `**Type**:` (hard error if missing) and that any `**Depends on**:` dependencies are complete. For each phase, invokes `Skill(code, <type>)` — instruction-loading in the orchestrator, not subagent dispatch. Commits once per phase. The terminal review phase runs an evaluate-fix loop combining static verification (`code review`) and orchestrator-driven behavioral verification; on non-convergence, halts without committing.
MUST read operations/execute.md before executing.

## Combined Operations

- **"plan this"** / **"break this down"** / **"create plan"** / **"phase plan"** / **"turn this Brief into phases"** → Run Create
- **"continue implementing"** / **"implement plan"** / **"execute plan"** / **"run plan"** / **"work through the plan"** → Run Execute
- **"plan and execute"** / **"plan and build"** → Run Create, then Execute on the resulting plan.md
- **"tracer bullets"** / **"vertical slices"** → Run Create (these terms refer to the planning method)

## References

- references/phase-templates.md — Per-type starter acceptance criteria and phase-title conventions for each of the four phase types (`write`, `test`, `review`, `benchmark`).
