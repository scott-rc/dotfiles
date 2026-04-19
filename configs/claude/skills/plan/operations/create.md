# Create Plan

Turn a Brief-populated plan file into a phased plan. Input is `tmp/<name>/plan.md` with its `## Brief` section populated (produced by `prd` or `code architect <target>`); output is the SAME file with `## Phase N` blocks appended and `**Type**:` metadata on each phase.

Plans are never authored by humans. The Brief is written by a seeder skill (`prd` for features, `code architect` for refactors). This operation reads the Brief, slices it into tracer-bullet phases, assigns a type to each, pulls templated acceptance criteria from `references/phase-templates.md`, and appends a default terminal review phase whose criteria derive from the Brief's `### Review Criteria` section.

## Process

### 1. Locate the plan file

If the user provided a path, use it. Otherwise search for Brief-only plan files (plan files with `## Brief` populated but no `## Phase N` sections) in `./tmp/*/plan.md`. If exactly one is found, use it. If multiple are found, present them as options. If none are found, ask the user for the path.

If the target file already has `## Phase N` sections, STOP and report — re-phasing a phased plan requires explicit user confirmation. Ask before proceeding.

### 2. Identify the Brief shape

Two input Brief shapes are supported:

- **Architect-seeded Brief** — `## Brief` contains sections: Problem, Proposed Interface, Dependency Strategy, Testing Strategy, Implementation Recommendations, Rejected Alternatives, Review Criteria. Produced by `code architect <target>`.
- **PRD-seeded Brief** — `## Brief` contains feature-spec content: user stories, behavior spec, constraints, open questions, Review Criteria. Produced by `prd`.

Detect which shape the Brief has by scanning for section headers. Use the shape to guide phase slicing.

### 3. Explore the codebase (if not already done in this session)

Understand the current architecture, existing patterns, and integration layers. The Brief describes *what* to build; the codebase tells you *how* to slice it into phases that keep tests green at each checkpoint.

### 4. Extract durable decisions

Scan the Brief for decisions unlikely to change during implementation:

- Module boundaries / interface names (from Proposed Interface in architect Briefs)
- Route patterns, schema, data models (from PRD Briefs)
- Dependency strategy (every Brief)
- Test boundary (every Brief)

These stay implicit unless they'd be useful at-a-glance during execution. Do NOT add a separate `## Architectural decisions` section unless the Brief explicitly calls for cross-phase decisions not captured in the Brief itself; a single-refactor plan's Brief already contains them.

### 5. Slice into tracer-bullet phases

Apply vertical-slice rules:

<vertical-slice-rules>
- Each slice delivers a narrow but COMPLETE path through every layer (schema, API, UI, tests)
- A completed slice is demoable or verifiable on its own
- Prefer many thin slices over few thick ones — but only where each slice leaves the tree green
- Do NOT include specific file names, function names, or implementation details that are likely to change as later phases are built
- DO include durable decisions: interface names, data model names, route paths
</vertical-slice-rules>

For refactor Briefs, the Brief's `### Implementation recommendations` section may suggest a phase sequence; use it as a starting point and adjust as needed.

### 6. Assign a Type to each phase

Every phase MUST have `**Type**: <write|test|review|benchmark>`. No defaults — this is a hard requirement.

- `write` covers most phases: behavior changes, bug fixes, refactoring, config/glue.
- `test` covers pure test-coverage work (backfills, mutation testing).
- `benchmark` covers performance-target phases.
- `review` is the terminal phase (see step 8).

See `references/phase-templates.md` for per-type conventions and starter criteria.

### 7. Record the Base SHA and any cross-plan dependencies in the plan header

Record the current git HEAD as the plan's **Base SHA** — the commit from which this plan's work begins. `plan execute` records per-phase commit SHAs; `plan review` uses `<Base SHA>..HEAD` as the plan's commit range to distinguish phase commits from scope-creep commits.

Capture the SHA with `git rev-parse HEAD`. Write it on its own line in the plan header, immediately after the title:

```markdown
# Plan: <name>

**Base**: <full-sha>
```

Additionally, if the Brief states or implies the plan presupposes another plan has been executed (e.g. "depends on the search refactor landing first"), add a `**Depends on**:` line below the Base line:

```markdown
# Plan: <name>

**Base**: <full-sha>
**Depends on**: tmp/<other-plan-name>/plan.md
```

Multiple `**Depends on**:` lines are supported — one path per line. `plan execute` will refuse to start if any dependency plan is not complete.

If the working tree has uncommitted changes at plan-create time, still capture HEAD; note to the user that the base is HEAD, not the working tree — any uncommitted changes will either be included in phase 1's commit or stay uncommitted and show up as a Scope-creep item at review time.

### 8. Append a default terminal review phase

Every plan ends with a review phase UNLESS the user explicitly opts out (by adding `**No review**: <rationale>` in the plan header or deleting the phase after creation).

The review phase's acceptance criteria are derived from the Brief's `### Review Criteria` section, which splits into `**Code**:` (static) and `**Behavior**:` (behavioral). Copy each bullet verbatim from the Brief into the phase's acceptance criteria as checkbox items, preserving the Code / Behavior split.

If the Brief has no Review Criteria section, use the defaults from `references/phase-templates.md` under the `Type: review` section.

### 9. Quiz the user

Present the proposed phase breakdown as a numbered list. For each phase show:

- **Title**: short descriptive name
- **Type**: the assigned phase type
- **Covers**: one-line description of the slice

Ask the user:

- Does the granularity feel right? (too coarse / too fine)
- Are the Type assignments correct?
- Should any phases be merged, split, or re-typed?

Iterate until the user approves.

### 10. Write the phases into the plan file

Append phase blocks to the existing plan file (below the Brief). Do NOT overwrite the Brief. Use the template below for each phase.

<phase-template>
## Phase N: <Title>

**Type**: <write | test | review | benchmark>

### What to build

A concise description of this vertical slice. Describe the end-to-end behavior, not layer-by-layer implementation.

### Acceptance criteria

- [ ] <starter criteria from references/phase-templates.md, customized per Brief>
- [ ] <phase-specific behavior criteria derived from the Brief>
</phase-template>

The review phase uses the review-specific template from `references/phase-templates.md` (with the Code / Behavior split).

### 11. Report

Summarize: plan file path, number of phases, phase types, any `**Depends on**:` header, next step (`plan execute <plan-path>`).
