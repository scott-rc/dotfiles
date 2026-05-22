# Review Plan

Post-execute retrospective. Survey a plan file's execution state — commits landed, Unchecked criteria, halt-findings, and phase-findings — and write a Retrospective section back into the plan file. For each Open item, propose a Resolution (Fixup phase, New plan, or Acknowledgment) and let the user confirm per-item. On confirmation of any Fixup phases, prompt to Auto-continue into `plan execute`.

## Terminology

This operation introduces distinct terms. Docs MUST use the two-word forms; bare "review" and bare "findings" are ambiguous in this skill.

- **plan review** — this operation.
- **review phase** — a Phase whose `**Type**:` is `review`.
- **code review** — the `code` skill's review operation, invoked by a review phase.
- **Halt-findings section** (`## Review findings`) — written by `plan execute` when a review phase halts on non-convergence.
- **Phase-findings section** (`## Phase N review-phase findings`) — written inside a review phase that converged but wants to narrate what it fixed or deferred.
- **Retrospective section** (`## Retrospective — <date>`) — written by this operation.
- **Open item** — anything surfaced by this operation as needing resolution. NOT called "finding" inside a Retrospective.
- **Deferral** — an Unchecked criterion whose prose contains a deferral signal.
- **Oversight** — an Unchecked criterion with no deferral signal.
- **Fixup phase** / **New plan** / **Acknowledgment** — the three Resolutions.

See `UBIQUITOUS_LANGUAGE.md` if one exists in the working directory (the `ubiquitous-language` skill writes it there).

## Entry points

`plan review` is invoked in three situations. All three run the same process below.

1. **Tail of every `plan execute`.** `plan execute` invokes `Skill(plan, review)` as its final step, regardless of outcome (clean completion, halt on non-convergence, or partial skip). This is the dominant entry point.
2. **Explicit user invocation.** User runs `plan review [path]` directly. Handles ad-hoc inspection and re-runs.
3. **Halt handoff.** When a review phase halts on non-convergence inside `plan execute`, execute's halt-handling delegates to this operation instead of prompting the user inline. (Mechanically identical to #1 — same tail invocation — but listed separately because it replaces behavior that used to live in `execute.md`.)

## Process

### 1. Locate the plan file

If the user provided a path, use it. Otherwise search for plan files in `./tmp/*/plan.md`. If exactly one is found, use it. If multiple are found, present them as options. If none are found, ask the user for the path.

### 2. Validate precondition

**Plan must have execution history.** If no phase has a `**Commit**:` line filled in — and no review artifacts (`## Review findings`, `## Phase N review-phase findings`) exist — refuse. Report: "Plan has no execution history; `plan review` is a post-execution operation. Did you mean `plan execute`?"

One exception: a plan with only a halt-findings section counts as having execution history (phases partially ran; the halt interrupted commit recording).

### 3. Survey the input surface

Collect data from the plan file and git. Do all of these in parallel where possible.

#### 3a. Phases and their state

For each Phase, capture:

- Phase number, title, Type.
- `**Commit**:` SHA if present (written by `plan execute`).
- `**PR**:` URL if present (written by `plan execute` after `Skill(git, push)`).
- `**Skipped**: <rationale>` if present.
- All acceptance criteria with their state: `- [ ]` (unchecked), `- [x]` (checked), `- [s]` (skipped).
- For each Unchecked criterion, capture the full line text (including any inline prose after the criterion text).

#### 3b. Review artifacts

Scan for these sections at the tail of the file:

- `## Review findings` — halt-findings section from a halted review phase.
- `## Phase N review-phase findings` — phase-findings section from a converged review phase.
- `## Retrospective — <date>` — prior Retrospective sections from earlier `plan review` runs. These inform (don't get replaced by) the new Retrospective.

#### 3c. Commit range

- Read the plan's `**Base**:` header if present. Compute `git log <base>..HEAD` to list all commits in the Plan commit range.
- Cross-reference each commit SHA against the `**Commit**:` line of every Phase. Commits in the range NOT tied to a Phase are Scope-creep commits — flag them.
- If any Phase's `**Commit**:` SHA isn't reachable in git (rewritten history, amended away), note it. Do not refuse — the plan file is the source of truth; git is just a data source.
- If `**Base**:` is missing (plan predates the convention), skip the range check. Use per-phase SHAs only; Scope-creep detection is unavailable.

#### 3d. Brief's Review Criteria cross-check

Compare each bullet in the Brief's `### Review Criteria` (both `**Code**:` and `**Behavior**:`) against the terminal review phase's acceptance criteria. Any Brief criterion not represented in any phase is missed-coverage; surface as an Open item.

### 4. Classify each candidate Open item

For each signal from step 3, classify:

#### 4a. Unchecked criteria in phases otherwise marked complete

An Unchecked criterion on a Phase whose other boxes are mostly `- [x]` (or whose subsequent phases have been executed past it) is an Open item.

Classify by scanning the full line text and any adjacent prose:

- **Deferral** if the line matches (case-insensitive) any of: `defer`, `deferred`, `pending`, `TBD`, `not measured`, `not directly measured`, `out of scope`, `deferred to`, `deferred without`.
- **Oversight** otherwise.

Unchecked criteria on a Phase with NO other checked boxes are not Open items — that phase simply never ran, which is execute's concern, not review's.

#### 4b. Halt-findings section entries

Every entry in a `## Review findings` section is an Open item. Parse the `file:line — severity — sentence` format; severity (Blocking / Improvement / Suggestion) influences default Resolution.

#### 4c. Phase-findings section entries

Scan for explicit deferrals — e.g. "One Suggestion deferred without fixing" with a following bullet list. These are Open items. Resolved findings mentioned in the same section (e.g. "fixed in the review loop") are NOT Open items; they go in the Summary instead.

#### 4d. Missed Review Criteria

Any Brief Review Criteria bullet with no corresponding phase acceptance criterion is an Open item, classified as **Oversight** (no deferral prose to classify otherwise).

### 5. Propose a default Resolution per Open item

For each Open item, compute a default Resolution the user will confirm or redirect.

| Item classification | Default Resolution |
|---------------------|--------------------|
| Deferral, rationale mentions system noise / benchmarks / fixtures / "quiet system" / performance measurement | Fixup phase, Type: `benchmark` |
| Deferral, rationale mentions test coverage / characterization / mutation | Fixup phase, Type: `test` |
| Deferral, rationale mentions audit / sweep / scan / reconciliation | Fixup phase, Type: `audit` |
| Deferral, rationale mentions API changes / refactors / "broader" / "out of scope" | New plan |
| Deferral, other rationale | Fixup phase, Type: `write` |
| Oversight | Ask the user — no default |
| Halt-findings entry, Blocking or Improvement | Fixup phase, Type: `write` |
| Halt-findings entry, Suggestion | New plan |
| Phase-findings Suggestion marked "deferred" | New plan |
| Missed Review Criterion | Ask the user — could be a bug in `plan create`'s Review Criteria cross-check, or a legitimate scope trim |

Rationale: narrow verifications stay in the current plan (Fixup phase); scope-creep-flagged items spin out (New plan); items with no deferral signal need user input.

### 6. Quiz the user per item

Present Open items as a numbered list. For each show:

- Source (phase number + criterion text, or findings-section entry).
- Classification (Deferral / Oversight / Halt-finding / Phase-finding / Missed-criterion).
- Default Resolution (Fixup phase with proposed Type / New plan / Acknowledgment).

For each item ask: accept default, change Resolution, or change Type (if Fixup phase). Accept multi-item shortcuts: "accept all defaults," "all to Acknowledgment," etc.

For Fixup phases, additionally gather:

- Phase title (propose from the criterion text; user can override).
- What-to-build description (propose a one-line derivation; user can edit).

For New plans, propose a name (derived from the Open item) and confirm. Do NOT seed the new plan's Brief — that's `plan create` (for features) or `code architect` (for refactors). `plan review` only creates the entry in the Retrospective; the user runs the seeder separately.

### 7. Write the Retrospective section

Append a new section to the plan file. NEVER replace an existing `## Retrospective — <date>` section — each run gets its own dated entry, appended below prior ones.

Date format: `YYYY-MM-DD` in ET. If another Retrospective exists with today's date, suffix: `## Retrospective — 2026-04-19 (2)`.

Retrospective structure:

```markdown
## Retrospective — <date>

### Summary

- Shipped: <N> phases (phases <list>), <N> commits in range <base>..<head-sha>
- PRs opened: <list of `**PR**:` URLs from phase headers, omit line if none>
- Review phase: <converged | halted | n/a> <brief note>
- Open items: <N> → <N> Fixup phase(s), <N> New plan(s), <N> Acknowledgment(s)
- Scope-creep commits: <N> (<shas> if any)

### Open items

| # | Source | Classification | Resolution |
|---|--------|----------------|------------|
| 1 | Phase 7 bench criterion | Deferral | Fixup: Phase <N+1> (benchmark) |
| 2 | ... | ... | ... |

### Acknowledgments

- <open item + rationale> — if the user chose Acknowledgment for anything

### New plans spawned

- `tmp/<name>/plan.md` — <one-line scope description>
```

Omit sub-sections that would be empty (e.g. no Acknowledgments → skip that heading).

### 8. Append Fixup phases

For each confirmed Fixup phase, append a Phase block after the last existing phase AND after any existing `## Phase N review-phase findings` section (file order: Brief → Phases 1..N → phase-findings sections → new Fixup phases → Retrospective sections).

Number sequentially from the last existing phase. Use the standard phase template from `references/phase-templates.md` matching the confirmed Type. Acceptance criteria: start with the starter criteria for the Type, then add one criterion that explicitly closes the Open item (e.g. `- [ ] Phase 7's bench-verification criterion now passes: <restate the criterion>`).

Fixup phases do NOT get `**Commit**:` metadata — that's written by `plan execute` when it runs them.

### 9. Prompt to Auto-continue

If any Fixup phases were appended, ask the user:

> N Fixup phase(s) appended. Continue into `plan execute` now?

On yes, invoke `Skill(plan, execute)` on the current plan file. Execute finds the resume point (first Fixup phase), runs from there, and — per the always-handoff rule — will invoke `plan review` again at its tail.

On no, report the plan file path and the next step: "Run `plan execute <path>` when ready."

If no Fixup phases were appended (all items resolved to New plan or Acknowledgment, or no Open items at all), skip the prompt.

### 10. Report

One-paragraph summary: plan path, Retrospective section header, number of Fixup phases appended, number of New plans spawned, number of Acknowledgments, whether Auto-continue was triggered.

## Rules

- **Never commit.** `plan review` edits only `tmp/<name>/plan.md`, which is gitignored. The retrospective is a local-workspace artifact; `plan review` runs are invisible in git history.
- **Never replace a prior Retrospective section.** Append a new one. Earlier Retrospectives stand as historical record of prior decisions.
- **Never auto-classify Oversights.** An Unchecked criterion without a deferral signal always prompts the user — there's no safe default between "bug" and "pending work."
- **Never seed a New plan's Brief.** Record the spawn in the Retrospective, report the path to the user. The user invokes `plan create` (for features) or `code architect` (for refactors) to seed the new plan.
- **Never skip the Auto-continue prompt** when Fixup phases were appended. Context-remaining is not a reliable signal; always ask.
- **Every `plan execute` ends with `plan review`.** Clean completion, halt, or full skip — same handoff. On a converged plan with zero Open items, the Retrospective is a one-line summary and the operation exits without further action.

## Edge cases

- **Plan never executed** — refuse per step 2.
- **Base SHA missing** — proceed without commit range; per-phase SHAs still work; Scope-creep detection unavailable; note in Summary.
- **Phase commit SHA unreachable in git** — list the missing SHAs in the Summary; don't refuse. Plan file is the source of truth.
- **Zero Open items** — write a one-line Summary, no tables, exit. The Retrospective is a record of "reviewed and clean."
- **All items are Oversights** — expected if a plan halts mid-execution on an Oversight; prompt the user per step 6 for each.
- **Previously-spawned New plan's path already exists** — warn and ask user to confirm a different name, or confirm overwrite (rare — requires a plan with the same derived name).
- **Re-running on a Retrospective that's already current** — append a second dated Retrospective. If the state is identical to the prior run, the new Retrospective will be near-identical. No special dedupe; user can delete by hand.
