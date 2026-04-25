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

Every phase MUST have `**Type**: <write|test|review|benchmark|audit>`. No defaults — this is a hard requirement.

- `write` covers most phases: behavior changes, bug fixes, refactoring, config/glue.
- `test` covers pure test-coverage work (backfills, mutation testing).
- `benchmark` covers performance-target phases.
- `review` is the terminal phase (see step 8).
- `audit` covers "sweep the surface, surface findings, triage with user, apply approved fixes" workflows — docs audits, dependency audits, security audits, config audits. STUB-status Type; its spec is minimal and expected to be refined as it gets real use.

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

**Type**: <write | test | review | benchmark | audit>

### What to build

A concise description of this vertical slice. Describe the end-to-end behavior, not layer-by-layer implementation.

### Acceptance criteria

- [ ] <starter criteria from references/phase-templates.md, customized per Brief>
- [ ] <phase-specific behavior criteria derived from the Brief>
</phase-template>

The review phase uses the review-specific template from `references/phase-templates.md` (with the Code / Behavior split).

### 11. Dispatch fresh-eyes review subagent

Before reporting completion, dispatch an **Opus** subagent with NO context from this conversation to review the plan for defects, risk, and weak criteria. Authoring eyes miss things fresh eyes catch — and at this stage the cost of finding a defect is one Edit, not a halted phase commit.

**Dispatch parameters:**

- `subagent_type: general-purpose`
- `model: opus` — required (cross-file verification benefits from the larger context window)
- Prompt is self-contained — the subagent has not seen this conversation

**Materials to point the subagent at (in the prompt):**

- The plan file (absolute path)
- Any supporting glossary: `UBIQUITOUS_LANGUAGE.md` at the repo root, if present
- The codebase root and the architectural-language reference at `~/.claude/skills/code/references/architecture-language.md`
- Subsystem `CLAUDE.md` files relevant to the Brief

**Tell the subagent NOT to re-litigate the Brief's design.** Design is locked at this stage. The subagent's job is to verify that the **phases deliver the Brief** without stranding work, breaking invariants, or missing call sites.

**Ask the subagent to look for:**

- **Cited fact verification** — line numbers, LOC counts, caller claims, "zero callers" claims. Have it `Read` the cited files and confirm the citations are accurate.
- **Missed call sites** — for each migration, does the phase's call-site list cover every consumer? `Grep` the codebase to verify.
- **Weak / unverifiable acceptance criteria** — bullets that someone could check off without doing the work, or that aren't testable.
- **Phase ordering risks** — does any phase depend on something a later phase delivers? Could a phase commit leave the tree red?
- **Hidden coupling** — does the plan touch a subsystem (search, e2e snapshots, service worker, design tokens, config) without saying so? Does it leave a subsystem broken that wasn't in scope?
- **Vocabulary slippage** — if the plan ran a renaming pass, does any prose still use old names? (Skip "Aliases to avoid" columns and "Flagged ambiguities" sections — those legitimately mention old terms.)
- **Risk not surfaced** — what would actually go wrong during execution that the plan doesn't mention? Examples: schema migration ordering, binding name mismatches, durable-state size limits, test-pool cold-start surprises, type re-export breakage in framework loaders, snapshot test flakiness from changed timing.
- **Scope drift** — Brief promising something a phase doesn't deliver, or a phase doing something the Brief doesn't authorize.
- **Test gaps** — flows the new tests don't cover that today's tests do (multi-user isolation, partial-failure paths, timing-related guards).
- **Audit phase coverage** (if present) — is the surface actually enumerable? Are categories distinct? Will it produce concrete findings or vague triage?

**Default scope guardrails — the subagent stays out of these unless a defect makes it necessary:**

- **Don't redesign.** The Brief's design is locked. EXCEPTION: flag a concrete defect in the chosen design — show the failure case (an invariant that can't hold, a call site that breaks). A working replacement isn't required; the goal is surfacing the defect.
- **Don't re-litigate rejected alternatives.** EXCEPTION: flag factual errors in the rejection reasoning that would change the verdict.
- **Don't expand this plan's scope.** EXCEPTION: flag coupling that crosses the plan's boundary — a subsystem outside this plan that's lockstep-coupled with one inside it. Surface as a Tier 3 follow-up candidate; do NOT propose adding it to the current plan.
- **Don't comment on prose style.** Style affecting verifiability of a criterion is already covered under "weak / unverifiable acceptance criteria."

**Output format the subagent should use:**

- **Tier 1 — defects the plan ships with** (the work won't complete correctly as written)
- **Tier 2 — improvements that materially help execution** (the work could complete but a future agent will hit friction)
- **Tier 3 — observations** (worth noting, no edit required)

For each finding: one-sentence description, the specific plan section / phase / criterion, and what to change. Cite file paths + line numbers when relevant. Cap the report at ~1500 words. Empty tiers say "none."

**After the subagent returns:**

1. Present the findings to the user grouped by tier, with your own read on each (agree / disagree / verify).
2. Apply approved Tier 1 + Tier 2 fixes inline via `Edit` on the plan file.
3. Skip Tier 3 unless the user asks otherwise.
4. If the subagent returned nothing in all three tiers, note that one-liner ("Fresh-eyes reviewer found no issues") and proceed.

The review is **mandatory** at the end of every `plan create` — there is no opt-out. A plan that's small enough to need no review is small enough that the review costs nothing.

### 12. Report

Summarize: plan file path, number of phases, phase types, any `**Depends on**:` header, next step (`plan execute <plan-path>`).
