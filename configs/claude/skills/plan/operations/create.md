# Create Plan

Produce a phased plan in `tmp/<name>/plan.md`. Two entry shapes:

- **No Brief yet** (new feature work) — interview the user to seed the `## Brief` section, then phase it.
- **Brief already populated** — read the existing Brief (e.g. seeded by `code architect <target>`) and append phases.

Output in both cases is the same file with `## Brief` populated and `## Phase N` blocks appended, each with `**Type**:` metadata.

Plans are never authored by humans. The Brief is either seeded inline by this operation (for features) or by `code architect` (for refactors). This operation reads or seeds the Brief, slices it into tracer-bullet phases, assigns a type to each, pulls templated acceptance criteria from `references/phase-templates.md`, and appends a default terminal review phase whose criteria derive from the Brief's `### Review Criteria` section.

## Process

### 1. Locate or seed the plan file

If the user provided a path to an existing plan file, use it.

Otherwise search for Brief-only plan files (`## Brief` populated but no `## Phase N` sections) in `./tmp/*/plan.md`:

- **Exactly one match** → use it.
- **Multiple matches** → present them as options to the user.
- **No matches** → switch to the no-Brief branch (step 2 will run the PRD interview to seed the Brief). Confirm with the user before launching the interview.

If the target file already has `## Phase N` sections, STOP and report — re-phasing a phased plan requires explicit user confirmation. Ask before proceeding.

### 2. Seed the Brief (no-Brief branch only)

If the plan file already has a populated `## Brief` section, skip this step and continue to step 3.

Otherwise, run the PRD-style interview to write the Brief:

1. **Ask for a long, detailed description** of the problem the user wants to solve and any potential ideas for solutions.

2. **Explore the repo** to verify the user's assertions and understand the current state of the codebase.

3. **Interview the user relentlessly** about every aspect of the plan until you reach shared understanding. Walk down each branch of the design tree, resolving dependencies between decisions one-by-one.

4. **Sketch the major modules** that will be built or modified. Actively look for opportunities to extract deep modules that can be tested in isolation.

   A deep module (as opposed to a shallow module) is one which encapsulates a lot of functionality in a simple, testable interface which rarely changes.

   Confirm with the user that these modules match their expectations. Confirm which modules they want tests written for.

5. **Write the Brief** to `./tmp/<name>/plan.md`. Create `./tmp/<name>/` if it doesn't exist. At this stage the file contains ONLY the `## Brief` section — phases come later. Use the template below.

<plan-brief-template>
# Plan: <name>

## Brief

### Problem Statement

The problem the user is facing, from the user's perspective.

### Solution

The solution to the problem, from the user's perspective.

### User Stories

A LONG, numbered list of user stories. Each user story in the format:

1. As an <actor>, I want a <feature>, so that <benefit>

<user-story-example>
1. As a mobile bank customer, I want to see balance on my accounts, so that I can make better informed decisions about my spending
</user-story-example>

This list should be extensive and cover all aspects of the feature.

### Implementation Decisions

A list of implementation decisions that were made. Include:

- The modules that will be built/modified
- The interfaces of those modules that will be modified
- Technical clarifications from the developer
- Architectural decisions
- Schema changes
- API contracts
- Specific interactions

Do NOT include specific file paths or code snippets — they may go stale quickly.

### Testing Decisions

A list of testing decisions. Include:

- What makes a good test for this feature (test observable behavior, not implementation details)
- Which modules will be tested
- Prior art for the tests (similar types of tests already in the codebase)

### Out of Scope

Things explicitly out of scope for this plan.

### Further Notes

Any further notes about the feature.

### Review Criteria

Criteria that the terminal review phase will check. Split into static code checks and behavioral checks.

**Code**:
- [one bullet per static criterion — e.g. "No feature-flag leakage", "Module interfaces match the Brief", "Test coverage ≥ X%", "Lint clean"]

**Behavior**:
- [one bullet per behavioral criterion — each user-visible behavior described in the user stories becomes a reviewable check, plus any regression surfaces worth exercising]
</plan-brief-template>

### 3. Identify the Brief shape

Two input Brief shapes are supported:

- **Architect-seeded Brief** — `## Brief` contains sections: Problem, Proposed Interface, Dependency Strategy, Testing Strategy, Implementation Recommendations, Rejected Alternatives, Review Criteria. Produced by `code architect <target>`.
- **PRD-seeded Brief** — `## Brief` contains feature-spec content: Problem Statement, Solution, User Stories, Implementation Decisions, Testing Decisions, Out of Scope, Further Notes, Review Criteria. Produced by step 2 of this operation (or written by hand in the same shape).

Detect which shape the Brief has by scanning for section headers. Use the shape to guide phase slicing. (If you just seeded the Brief in step 2, it's PRD-seeded.)

### 4. Explore the codebase (if not already done in this session)

Understand the current architecture, existing patterns, and integration layers. The Brief describes *what* to build; the codebase tells you *how* to slice it into phases that keep tests green at each checkpoint. (If the no-Brief branch ran in step 2, the codebase has already been explored — skip.)

### 5. Extract durable decisions

Scan the Brief for decisions unlikely to change during implementation:

- Module boundaries / interface names (from Proposed Interface in architect Briefs)
- Route patterns, schema, data models (from PRD Briefs)
- Dependency strategy (every Brief)
- Test boundary (every Brief)

These stay implicit unless they'd be useful at-a-glance during execution. Do NOT add a separate `## Architectural decisions` section unless the Brief explicitly calls for cross-phase decisions not captured in the Brief itself; a single-refactor plan's Brief already contains them.

### 6. Slice into tracer-bullet phases

Apply vertical-slice rules:

<vertical-slice-rules>
- Each slice delivers a narrow but COMPLETE path through every layer (schema, API, UI, tests)
- A completed slice is demoable or verifiable on its own
- Prefer many thin slices over few thick ones — but only where each slice leaves the tree green
- Do NOT include specific file names, function names, or implementation details that are likely to change as later phases are built
- DO include durable decisions: interface names, data model names, route paths
</vertical-slice-rules>

For refactor Briefs, the Brief's `### Implementation recommendations` section may suggest a phase sequence; use it as a starting point and adjust as needed.

### 7. Assign a Type to each phase

Every phase MUST have `**Type**: <write|test|review|benchmark|audit>`. No defaults — this is a hard requirement.

- `write` covers most phases: behavior changes, bug fixes, refactoring, config/glue.
- `test` covers pure test-coverage work (backfills, mutation testing).
- `benchmark` covers performance-target phases.
- `review` is the terminal phase (see step 9).
- `audit` covers "sweep the surface, surface findings, triage with user, apply approved fixes" workflows — docs audits, dependency audits, security audits, config audits. STUB-status Type; its spec is minimal and expected to be refined as it gets real use.

See `references/phase-templates.md` for per-type conventions and starter criteria.

### 8. Record the Base SHA and any cross-plan dependencies in the plan header

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

### 9. Append a default terminal review phase

Every plan ends with a review phase UNLESS the user explicitly opts out (by adding `**No review**: <rationale>` in the plan header or deleting the phase after creation).

The review phase's acceptance criteria are derived from the Brief's `### Review Criteria` section, which splits into `**Code**:` (static) and `**Behavior**:` (behavioral). Copy each bullet verbatim from the Brief into the phase's acceptance criteria as checkbox items, preserving the Code / Behavior split.

If the Brief has no Review Criteria section, use the defaults from `references/phase-templates.md` under the `Type: review` section.

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

### 11. Iterated fresh-eyes review

Before reporting completion, run an **iterated** fresh-eyes review. Each round dispatches an Opus subagent with no prior context, returns findings, and informs the next round of edits. Loop until the plan converges (a round returns no Tier 1 and no Tier 2 findings) or a hard cap of **4 rounds** is reached, whichever comes first.

Why iterate: in practice each round catches different kinds of defects.

- **Round 1** finds the obvious gaps: cited fact errors, missed call sites, weak criteria, surface-level vocabulary slippage.
- **Round 2** finds second-order issues that depend on round-1 fixes — e.g. signature changes that cascade to test files, criteria that cite renamed methods.
- **Round 3** catches contradictions introduced by recent edits — heavy editing creates new drift, like an interface signature asserted differently in two sections, or a return shape that two criteria disagree on.
- **Round 4** is usually convergent. If it still finds Tier 1 issues, the plan has structural problems worth surfacing to the user before stopping.

Stopping early matters too: if round N returns empty Tier 1 and Tier 2, do NOT dispatch round N+1. Tier 3 noise alone doesn't justify another round.

**Dispatch parameters per round:**

- `subagent_type: general-purpose`
- `model: opus` — required (cross-file verification benefits from the larger context window)
- Prompt is self-contained — the subagent has not seen this conversation OR prior rounds

**Materials to point each subagent at (in the prompt):**

- The plan file (absolute path)
- Any supporting glossary: `UBIQUITOUS_LANGUAGE.md` at the repo root, if present
- The codebase root and the architectural-language reference at `~/.claude/skills/code/references/architecture-language.md`
- Subsystem `CLAUDE.md` files relevant to the Brief

**Round-specific framing in the prompt header.** Tell the subagent which round this is and what's already been done — sets the bar appropriately:

- **Round 1:** "Bring fresh eyes. The plan has had no review yet."
- **Round 2:** "The plan has been through one round of fresh-eyes review and the easy stuff was fixed. Look for second-order issues, missed call sites in the migrations the round-1 edits touched, and inconsistencies the edits introduced."
- **Round 3:** "The plan has been through two rounds. Heavy editing creates drift — look for contradictions across sections (interface signatures asserted differently, criteria that disagree on a return shape, dead parameter survivals). Empty tiers are entirely valid here."
- **Round 4:** "The plan has been through three rounds and is approaching convergence. Only flag genuine defects you're confident about. Empty tiers are the expected result; don't manufacture findings."

**Tell every subagent NOT to re-litigate the Brief's design.** Design is locked at this stage. The subagent's job is to verify that the **phases deliver the Brief** without stranding work, breaking invariants, or missing call sites.

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

For each finding: one-sentence description, the specific plan section / phase / criterion, and what to change. Cite file paths + line numbers when relevant. Cap the report at ~1500 words. Empty tiers say "none." Do not manufacture findings to fill a tier.

**After each round returns:**

1. **Verify each finding before applying.** Read the cited file/lines, run the cited grep, confirm the issue exists. Subagents occasionally misread (e.g. claim a function has zero callers when it has test-only callers; misread a `vi.spyOn` target). Wrong findings, applied uncritically, degrade the plan and cause the next round to find new contradictions.
2. Present findings to the user grouped by tier with your own read on each (agree / disagree / verify).
3. Apply approved Tier 1 + Tier 2 fixes inline via `Edit` on the plan file.
4. Skip Tier 3 unless the user asks otherwise.
5. **Check the stop condition.** If both Tier 1 and Tier 2 came back empty, the plan has converged — exit the loop. Otherwise, dispatch the next round (up to round 4).

**Stop condition:**

- **Convergence:** A round returns no Tier 1 and no Tier 2 findings. (Empty Tier 3 is not required.)
- **Hard cap:** 4 rounds completed.

When the loop ends, note the outcome to the user (e.g. "Fresh-eyes review converged after 3 rounds" or "Reached the round-4 cap; remaining Tier 1 findings are listed above for your review"). If the cap was reached with unresolved Tier 1 findings, prefer surfacing them rather than silently proceeding — they signal structural plan issues worth addressing before `plan execute`.

The iterated review is **mandatory** at the end of every `plan create` — there is no opt-out. A plan that's small enough to need no review is small enough that the review costs nothing.

### 12. Report

Summarize: plan file path, number of phases, phase types, any `**Depends on**:` header, next step (`plan execute <plan-path>`).
