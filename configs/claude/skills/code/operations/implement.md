# Implement

Run a full TDD → code review convergence loop: build test-first, then review-fix until converged. Mutation testing is available as an opt-in hardening step.

## Instructions

### Phase 1 — Build with TDD

1. **Clarify the implementation type**:
   Determine whether this is new behavior (feature, endpoint, function) or a bug fix. If ambiguous, ask focused questions before proceeding.

2. **Follow the TDD operation**:
   - New behavior → Follow operations/new-feature.md
   - Bug fix → Follow operations/fix-bug.md

   Track all files created or modified during this phase for use in Phases 2 and 3.

3. **Confirm Phase 1 complete**:
   Verify all tests pass. Capture the file list before proceeding.

### Phase 2 — Harden with Mutation Testing (opt-in)

4. **Decide whether to run mutation testing**:
   Skip by default. Run mutation testing only when:
   - User explicitly requests it ("with mutation testing", "run MT", "harden")
   - Code has complex branching logic where TDD alone may leave weak assertions

   If skipping, proceed directly to Phase 3.

5. **Follow the Mutation Testing operation**:
   Follow operations/mutate.md, scoping it to the files tracked in step 2.

6. **Report mutation testing outcome**:
   Report to the user: score before/after, survivors killed. Confirm all tests pass before proceeding to review.

### Phase 3 — Review-Fix Convergence Loop

7. **Follow the Review operation**:
   Follow operations/review.md on all files written or modified across Phases 1 and 2. Skip review.md's fix-plan offer (step 13) — this phase owns the convergence loop.

8. **Drive the fix-review loop**:
   This phase owns the convergence loop (up to 4 iterations):
   - Blocking and Improvement findings — fix immediately. Escalate only when the fix has multiple plausible approaches and no available context disambiguates.
   - Suggestions — fix if quick (fewer than 3 per file); otherwise record and include in the Phase 4 report.
   - Recurring findings — if the same finding recurs after a fix attempt, escalate to the user or record as "acknowledged, not addressed" with rationale.
   - Fix findings, then re-follow operations/review.md on the same file set.
   - Stop when review reports no remaining findings or 4 iterations are reached. At max iterations, hand off remaining findings to the Phase 4 report.

### Phase 4 — Report

9. **Report results**:
   MUST report to the user:
   - Implementation summary: tests written, behaviors covered, TDD approach used (red-green-refactor or regression-first), files created/modified
   - Mutation testing: score before/after (if run)
   - Review cycles completed and final status (converged or remaining findings with categories)
   - Deferred items: edge cases noted during TDD, integration-level mutation survivors, unaddressed suggestions
