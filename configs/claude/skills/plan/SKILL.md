---
name: plan
description: Create multi-phase implementation plans from design documents (PRDs or refactor RFCs) using tracer-bullet vertical slices, and execute them with commit checkpoints. Use when user wants to break down a PRD or RFC, create an implementation plan, execute a plan, continue implementing, or mentions "tracer bullets" or "vertical slices".
argument-hint: "[create | execute] [path]"
---

# Plan

Create and execute phased implementation plans. Works with design documents (PRDs from the `prd` skill, or refactor RFCs from `code architect`). Plans use tracer-bullet vertical slices; execution orchestrates the plan phase-by-phase with commit checkpoints.

## Operations

### Create
Break a design document (PRD or RFC) into a phased implementation plan using vertical slices. Output is `plan.md` saved next to the source design document.
MUST read operations/create.md before executing.

### Execute
Orchestrate multi-phase plan execution with commit checkpoints between phases, skip tracking, and interactive UI verification. Routes each phase's implementation to the `code` skill's Write mode (TDD for new behavior/fixes, Apply for refactoring/config/glue).
MUST read operations/execute.md before executing.

## Combined Operations

- **"plan this"** / **"break this down"** / **"create plan"** / **"phase plan"** / **"turn this PRD/RFC into a plan"** → Run Create
- **"continue implementing"** / **"implement plan"** / **"execute plan"** / **"run plan"** / **"work through the plan"** → Run Execute
- **"plan and execute"** / **"plan and build"** → Run Create, then Execute on the resulting plan.md
- **"tracer bullets"** / **"vertical slices"** → Run Create (these terms refer to the planning method)
