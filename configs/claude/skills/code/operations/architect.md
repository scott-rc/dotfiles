# Architect

Design an architectural refactor. Identify shallow modules or friction points, generate parallel design alternatives, produce a plan file whose `## Brief` captures the chosen design. Does not implement. Hands off to `plan create` for phasing.

## Invocation modes

- **Discovery** (no args) — explore the codebase for candidates, present a numbered list, user picks one or many. Each pick becomes a separate architect invocation.
- **Targeted** (`code architect <target>`) — skip discovery; go straight to framing for the specified module, file, or area. Produces `tmp/<target>/plan.md` with a Brief populated.

## Instructions

### 1. Identify candidate (skip in targeted mode)

**Explore the codebase inline — do NOT delegate to a subagent.** Discovery is architectural judgment against verified ground truth; a subagent summary drops the numbers and cross-file detail that shape the design. Use Glob, Grep, and Read directly. If context budget is tight, say so and stop — do not smuggle delegation back in under the guise of "just one narrow Explore agent."

Look for friction points:

- Concepts scattered across many files
- Overly simple interfaces hiding complex implementations, or overly large interfaces hiding trivial ones
- Pure functions extracted solely to be testable
- Tightly-coupled modules with integration risk
- Untestable or hard-to-test sections

**Ground every claim.** Each candidate you surface must cite specific files (with line numbers where useful) and verified counts. No approximations like "~16 fields" when a Read or grep gives the exact number. If you're about to write a range or a hedge, run the tool call first.

Present findings as a numbered list, **ranked by architectural payoff, highest first, max 3 candidates, no minimum** (zero is a valid terminal output — "nothing stands out"). For each candidate:

- Which modules cluster together (with file paths and line references)
- Why they're coupled (shared types, ordering requirements, leaking state — with concrete evidence)
- Dependency category per [references/dependency-categories.md](../references/dependency-categories.md)
- Testing implications (what's currently tested; what's hard to test; what would be possible after deepening)

Do NOT propose specific interfaces yet. Ask the user to pick one or more candidates. Each picked candidate becomes a separate targeted invocation — there is no single roadmap file spanning multiple candidates.

### 2. Frame constraints

For the selected candidate (or the specified target in targeted mode), write a user-facing explanation covering:

- What any new interface must satisfy — inputs, outputs, invariants callers depend on
- Relevant dependencies, classified per [references/dependency-categories.md](../references/dependency-categories.md)
- Illustrative code sketches grounding the problem space (small snippets, not full designs)

Present this framing to the user before spawning design agents.

### 3. Dispatch 4 parallel design subagents

Each subagent receives:
- The framing from step 2
- ONE of the 4 design constraints below
- A required deliverable: interface signature, usage example, hidden complexity, dependency strategy, trade-offs

**Design constraints (3 fixed + 1 adaptive based on dependency category):**

| Slot | Constraint |
|---|---|
| 1 | **Minimal interface** — smallest viable API surface |
| 2 | **Maximum flexibility** — callers retain full control; nothing locked in |
| 3 | **Common-case optimization** — default path is effortless; rare paths remain possible |
| 4 | Adaptive: in-process → **maximum locality** (keep implementation compact and local); local-substitutable → **stand-in-first** (design around the test substitute); remote-but-owned → **ports-and-adapters**; true external → **mock-boundary** |

Do not prescribe the subagent type — the orchestrator selects at dispatch time based on the framing.

Run the 4 subagents in parallel.

### 4. Present designs; user picks

Surface all 4 designs side-by-side. For each: interface signature, usage example, hidden complexity, dependency strategy, trade-offs. User picks one (or directs a merge). Capture the 3 rejected designs with their constraint label and trade-offs — they go into the Brief's Rejected Alternatives section.

### 5. Write the plan file with Brief populated

Create `tmp/<target>/plan.md` (create the directory if needed). `<target>` is a short slug derived from the module or area (lowercase, kebab-case, max 40 chars). If the user invoked with an explicit `<target>` argument, use it as the slug.

The file contains ONLY the `## Brief` section at this stage. No phases. Use the template below.

<plan-brief-template>
# Plan: <name>

## Brief

### Problem

Describe the architectural friction:
- Which modules are shallow and tightly coupled
- What integration risk exists in the seams between them
- Why this makes the codebase harder to navigate and maintain

### Proposed interface

The chosen design:
- Interface signature (types, methods, params)
- Usage example showing how callers use it
- What complexity it hides internally

### Dependency strategy

Which category applies and how dependencies are handled (per [dependency-categories.md](../references/dependency-categories.md)):
- **In-process**: merged directly
- **Local-substitutable**: tested with [specific stand-in]
- **Ports & adapters**: port definition, production adapter, test adapter
- **Mock**: mock boundary for external services

### Testing strategy

Replace, don't layer:
- Old unit tests on the formerly-shallow modules are waste once boundary tests exist — delete them
- New tests live at the deepened module's interface boundary
- Tests assert observable outcomes through the public interface, not internal state
- Tests should survive internal refactors

Specifics:
- **New boundary tests to write**: [describe behaviors to verify]
- **Old tests to delete**: [list shallow-module tests that become redundant]
- **Test environment needs**: [local stand-ins or adapters required]

### Implementation recommendations

Durable architectural guidance — NOT coupled to current file paths:
- What the module should own (responsibilities)
- What it should hide (implementation details)
- What it should expose (the interface contract)
- Suggested phase sequence for `plan create` (e.g. "introduce type behind dual-track, migrate consumers, delete old fields"). Optional but useful — phase sequencing is ultimately `plan create`'s call.

### Rejected alternatives

The other 3 designs explored, with trade-offs and why the chosen design won.

#### Alternative A: [constraint name]
- Interface sketch
- Trade-offs
- Why rejected

#### Alternative B: [constraint name]
- Interface sketch
- Trade-offs
- Why rejected

#### Alternative C: [constraint name]
- Interface sketch
- Trade-offs
- Why rejected

### Review Criteria

Criteria `plan create` will turn into acceptance criteria on the terminal review phase. Split into static code checks and behavioral checks.

**Code**:
- [one bullet per static criterion specific to this refactor — e.g. "No old-module imports remain", "Public API is minimal", "Dependency direction matches the design"]

**Behavior**:
- [one bullet per behavioral criterion — user-visible behavior that must still work, regression surfaces to exercise]

For pure-internal refactors with no user-visible surface, the Behavior list can be a single smoke check ("Binary still builds and existing integration tests pass unchanged").
</plan-brief-template>

### 6. Report and hand off

The plan file is the terminal output at this step. Do NOT implement. Do NOT add phases — that's `plan create`'s job. Report to the user:

- Plan file location (`tmp/<target>/plan.md`)
- Summary of chosen design
- Next step: `plan create <plan-path>` to phase the Brief into a runnable plan.

If the user picked multiple candidates in discovery mode, repeat from step 2 for the next candidate (each targeted invocation produces its own plan file). Do NOT create a roadmap file spanning multiple candidates — there is no such concept. Cross-plan ordering is expressed via `**Depends on**:` headers on individual plan files.
