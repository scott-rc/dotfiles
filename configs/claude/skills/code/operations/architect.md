# Architect

Design an architectural refactor -- identify shallow modules or friction points, generate parallel design alternatives, produce a refactor RFC. Does not implement. Outputs `./tmp/refactor/<name>/rfc.md`.

## Invocation modes

- **Discovery** (no args) -- explore the codebase for candidates, present a numbered list, user picks one.
- **Targeted** (`code architect <target>`) -- skip discovery; go straight to framing for the specified module, file, or area.

## Instructions

### 1. Identify candidate (skip in targeted mode)

Dispatch a subagent to explore the codebase for friction points:

- Concepts scattered across many files
- Overly simple interfaces hiding complex implementations, or overly large interfaces hiding trivial ones
- Pure functions extracted solely to be testable
- Tightly-coupled modules with integration risk
- Untestable or hard-to-test sections

Present findings as a numbered list. For each candidate:
- Which modules cluster together
- Why they're coupled (shared types, ordering requirements, leaking state)
- Dependency category per [references/dependency-categories.md](../references/dependency-categories.md)
- Testing implications (what's currently tested; what's hard to test; what would be possible after deepening)

Do NOT propose specific interfaces yet. Ask the user to pick one candidate.

### 2. Frame constraints

For the selected candidate (or the specified target in targeted mode), write a user-facing explanation covering:

- What any new interface must satisfy -- inputs, outputs, invariants callers depend on
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
| 1 | **Minimal interface** -- smallest viable API surface |
| 2 | **Maximum flexibility** -- callers retain full control; nothing locked in |
| 3 | **Common-case optimization** -- default path is effortless; rare paths remain possible |
| 4 | Adaptive: in-process → **maximum locality** (keep implementation compact and local); local-substitutable → **stand-in-first** (design around the test substitute); remote-but-owned → **ports-and-adapters**; true external → **mock-boundary** |

Do not prescribe the subagent type -- the orchestrator selects at dispatch time based on the framing.

Run the 4 subagents in parallel.

### 4. Present designs; user picks

Surface all 4 designs side-by-side. For each: interface signature, usage example, hidden complexity, dependency strategy, trade-offs. User picks one (or directs a merge). Capture the 3 rejected designs with their constraint label and trade-offs -- they go into the RFC's Rejected Alternatives section.

### 5. Write the RFC

Create `./tmp/refactor/<name>/rfc.md` (create the directory if needed). `<name>` = short slug derived from the target module or area (lowercase, kebab-case, max 40 chars).

Use the template below.

<rfc-template>
# Refactor RFC: <name>

## Problem

Describe the architectural friction:
- Which modules are shallow and tightly coupled
- What integration risk exists in the seams between them
- Why this makes the codebase harder to navigate and maintain

## Proposed Interface

The chosen design:
- Interface signature (types, methods, params)
- Usage example showing how callers use it
- What complexity it hides internally

## Dependency Strategy

Which category applies and how dependencies are handled (per [dependency-categories.md](../../../configs/claude/skills/code/references/dependency-categories.md)):
- **In-process**: merged directly
- **Local-substitutable**: tested with [specific stand-in]
- **Ports & adapters**: port definition, production adapter, test adapter
- **Mock**: mock boundary for external services

## Testing Strategy

Replace, don't layer:
- Old unit tests on the formerly-shallow modules are waste once boundary tests exist -- delete them
- New tests live at the deepened module's interface boundary
- Tests assert observable outcomes through the public interface, not internal state
- Tests should survive internal refactors

Specifics:
- **New boundary tests to write**: [describe behaviors to verify]
- **Old tests to delete**: [list shallow-module tests that become redundant]
- **Test environment needs**: [local stand-ins or adapters required]

## Implementation Recommendations

Durable architectural guidance -- NOT coupled to current file paths:
- What the module should own (responsibilities)
- What it should hide (implementation details)
- What it should expose (the interface contract)
- How callers should migrate

## Rejected Alternatives

The other 3 designs explored, with trade-offs and why the chosen design won.

### Alternative A: [constraint name]
- Interface sketch
- Trade-offs
- Why rejected

### Alternative B: [constraint name]
- Interface sketch
- Trade-offs
- Why rejected

### Alternative C: [constraint name]
- Interface sketch
- Trade-offs
- Why rejected
</rfc-template>

### 6. Report and hand off

The RFC is terminal output. Do NOT implement. Report to the user:
- RFC location
- Summary of chosen design
- Next steps:
  - `plan create <rfc path>` to generate an implementation plan from the RFC
  - `git` skill to push the RFC as a GitHub issue if desired
