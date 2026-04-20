---
name: plan
description: Turn a Brief-populated plan file into phased work, execute it phase by phase with commit checkpoints, and retrospect on completion. Use when the user wants to break down a PRD or refactor Brief into phases, create an implementation plan, execute a plan, continue implementing, review a completed plan, or mentions "tracer bullets" or "vertical slices".
argument-hint: "[create | execute | review] [path]"
---

# Plan

Create, execute, and retrospect phased implementation plans. Plans live as single files at `tmp/<name>/plan.md` — one file per piece of work, from initial Brief through final Retrospective. Plans are seeded by `prd` (for features) or `code architect <target>` (for refactors); each seeder writes the Brief and hands off to `plan create`.

## Terminology

"Review" and "findings" are both overloaded in this skill. See `operations/review.md` for the canonical glossary; docs MUST use the unambiguous forms (plan review, review phase, code review, Halt-findings section, Phase-findings section, Retrospective section) and never the bare words.

## Operations

### Create
Read a Brief-populated plan file and append phases to it. Records the Base SHA (current `HEAD`) in the plan header. Assigns `**Type**:` per phase (`write`, `test`, `review`, `benchmark`, `audit`); pulls per-type starter acceptance criteria from `references/phase-templates.md`; appends a default terminal review phase whose criteria derive from the Brief's `### Review Criteria` section.
MUST read operations/create.md before executing.

### Execute
Run the plan phase by phase. Validates that every phase has `**Type**:` (hard error if missing) and that any `**Depends on**:` dependencies are complete. For each `write`, `test`, or `benchmark` phase, invokes `Skill(code, <type>)` — instruction-loading in the orchestrator, not subagent dispatch. `audit` is orchestrator-owned (no code-skill dispatch). Commits once per phase via `Skill(git, commit)` and records the resulting SHA on the phase's `**Commit**:` line. The terminal review phase runs an evaluate-fix loop combining static verification (`code review`) and orchestrator-driven behavioral verification; on non-convergence, halts without committing. As its final step — regardless of outcome — invokes `Skill(plan, review)` to produce the Retrospective.
MUST read operations/execute.md before executing.

### Review
Post-execute retrospective. Surveys the plan's execution state (per-phase commit SHAs, Unchecked criteria, Halt-findings, Phase-findings, and the Base SHA commit range), classifies each Open item, proposes a per-item Resolution (Fixup phase / New plan / Acknowledgment), writes a dated Retrospective section appended to the plan file, appends any confirmed Fixup phases, and offers to Auto-continue into `plan execute`. Writes only to the (gitignored) plan file; produces no commits of its own. Invoked automatically at the end of every `plan execute`, or explicitly by the user.
MUST read operations/review.md before executing.

## Combined Operations

- **"plan this"** / **"break this down"** / **"create plan"** / **"phase plan"** / **"turn this Brief into phases"** → Run Create
- **"continue implementing"** / **"implement plan"** / **"execute plan"** / **"run plan"** / **"work through the plan"** → Run Execute
- **"plan and execute"** / **"plan and build"** → Run Create, then Execute on the resulting plan.md
- **"tracer bullets"** / **"vertical slices"** → Run Create (these terms refer to the planning method)
- **"review the plan"** / **"plan retrospective"** / **"what's left on this plan"** / **"retrospect on the plan"** → Run Review

## References

- references/phase-templates.md — Per-type starter acceptance criteria and phase-title conventions for each of the five phase types (`write`, `test`, `review`, `benchmark`, `audit`).
