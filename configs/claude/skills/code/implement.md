# Implement

Run a full TDD → mutation testing → code review convergence loop: build test-first, harden with mutation testing, then review-fix until converged.

## Instructions

### Phase 1 — Build with TDD

1. **Clarify the implementation type**:
   Determine whether this is new behavior (feature, endpoint, function) or a bug fix. If ambiguous, ask focused questions before proceeding.

2. **Follow the TDD operation**:
   - New behavior → Follow [new-feature.md](new-feature.md)
   - Bug fix → Follow [fix-bug.md](fix-bug.md)

   Track all files created or modified during this phase for use in Phases 2 and 3.

3. **Confirm Phase 1 complete**:
   Verify all tests pass. Capture the file list before proceeding.

### Phase 2 — Harden with Mutation Testing (conditional)

4. **Assess mutation testing viability**:
   Skip mutation testing if any of these apply:
   - Code is trivial (config, glue, one-liner)
   - User opted out ("no mutation testing", "skip MT")
   - No meaningful mutations are possible (pure I/O wiring, constants only)

   If skipping, note the reason and proceed to step 6.

5. **Follow the Mutation Testing operation**:
   Follow [mutate.md](mutate.md), scoping it to the files tracked in step 2.

6. **Report Phase 2 status**:
   Report to the user: mutation testing outcome (score before/after, survivors killed) or reason skipped. Confirm all tests pass before proceeding to review.

### Phase 3 — Review-Fix Convergence Loop

7. **Follow the Review operation**:
   Follow [review.md](review.md) on all files written or modified across Phases 1 and 2.

8. **Drive the fix-review loop**:
   After review reports findings, implement.md owns the convergence loop (up to 4 iterations):
   - If Blocking findings are reported, MUST present them to the user before fixing.
   - Fix findings, then re-follow [review.md](review.md) on the same file set.
   - Stop when review reports no remaining findings or 4 iterations are reached.

### Phase 4 — Report

9. **Report results**:
   MUST report to the user:
   - Implementation summary: tests written, behaviors covered, TDD approach used (red-green-refactor or regression-first), files created/modified
   - Mutation testing: score before new tests (killed/total), score after (if survivors were killed), or reason skipped
   - Review cycles completed and final status (converged or remaining findings with categories)
   - Deferred items: edge cases noted during TDD, integration-level mutation survivors, unaddressed suggestions
