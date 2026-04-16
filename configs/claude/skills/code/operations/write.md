# Write Code

Load coding guidelines, write code, and verify it -- supports two modes: Apply (no TDD) and TDD (red-green-refactor for new behavior or bug fixes).

## Instructions

### Shared Setup

1. **Load coding guidelines**: Follow references/load-guidelines.md.

2. **Identify the mode**:
   - **Apply** -- no tests needed: refactoring under existing coverage, config, glue code, one-liners, or user opts out of TDD
   - **TDD** -- test-first for any behavior change, including new features AND bug fixes

   **Key test**: "does this change what the system does when it runs?" If yes → TDD mode, even if the underlying API already exists.

   Common misclassifications -- these are TDD mode, not Apply:
   - "ensure X uses URL params for state" -- adds server-side param reading + client-side sync (new behavior)
   - "make the API return pagination metadata" -- adds response fields (new behavior)
   - "add caching to the handler" -- adds cache logic (new behavior)
   - "sync state between X and Y" -- adds synchronization that didn't exist (new behavior)

   If the mode is ambiguous, ask the user before proceeding.

---

## Apply Mode

Write code without TDD -- for refactoring, config, glue, scripting, or when the user explicitly opts out of tests.

3. **Scope and write the code**: For broad maintainability or refactor requests, MUST identify the primary seam first, state the expected touched files, and call out unrelated working-tree changes before editing. Prefer the smallest clear edit sequence over whole-file rewrites; rewrite an entire file only when the structural simplification clearly outweighs diff churn. Apply the loaded preferences. When they conflict with existing project conventions (linter config, formatter, existing patterns), SHOULD follow project conventions.

4. **Self-check**: MUST verify code follows the loaded guidelines, paying special attention to naming, comments, nesting, error handling, and abstractions. SHOULD run the relevant formatter, linter, or static analysis used by the project when the change touches code those tools govern, even for refactors. For shell scripts, MUST run `shellcheck` and fix all warnings. If any violation is found, fix it and re-check. Iterate up to 3 times. If violations persist after 3 iterations, report remaining issues to the user.

5. **Run existing tests**: If the project has a test suite, run it (or the relevant subset) to confirm no regressions.

6. **Report results**: Present the code to the user with a summary of which preferences were applied, any conflicts resolved, touched files, and test suite status (pass/fail, number of tests run).

---

## TDD Mode

Build the behavior through the red-green-refactor cycle: write a failing test, make it pass with minimal code, then clean up. Works for new features AND bug fixes -- see the bug-fix sub-case below.

### Anti-pattern: horizontal slicing

**DO NOT write all tests first, then all implementation.** Batch-writing tests captures _imagined_ behavior, not _actual_ behavior -- the tests end up asserting on the shape of things (data structures, function signatures) rather than real user-facing outcomes, and they drift insensitive to real changes. Write ONE test, make it pass, then the next. Each test responds to what you learned from the previous cycle.

### Bug-fix sub-case

If the behavior being captured is a bug fix (not a new feature):

- Plan **one regression test** that captures the bug-free behavior -- not 2–5 cases
- The test MUST fail against current code for the right reason -- it must capture THIS bug, not a different problem
- **Keep the fix minimal.** Do NOT refactor or improve surrounding code during the fix -- defer any cleanup to a separate Apply-mode pass. The diff stays focused on the fix itself
- Skip step 4 (planning multiple cases) and go straight to step 5 (Red)

### Steps

3. **Understand the behavior**:
   Clarify what the behavior is, its inputs, outputs, and edge cases. For bug fixes: MUST identify expected behavior vs actual (broken) behavior, and reproduction steps. If the user's request is vague, ask focused questions before proceeding.

4. **Plan test cases (new-behavior sub-case only)**:
   **You can't test everything.** Explicitly confirm with the user which behaviors matter most -- focus testing effort on critical paths and complex logic, not every possible edge case. Draft 2–5 test cases covering the core behavior and important edge cases. Apply the special cases checklist from references/testing-guidelines.md to catch boundary conditions. Present the list to the user for confirmation before writing any code. Adjust based on their feedback.

5. **Red -- write a failing test**:
   Write the first test case. It MUST assert the expected behavior. The test SHOULD fail because the implementation doesn't exist yet (or, for bug fixes, because the bug is present).

6. **Verify failure**:
   MUST run the first test case to verify the harness works and the failure reason is correct. MAY skip for subsequent cases if the runner and assertion pattern are established.

7. **Green -- write minimal implementation**:
   Write the simplest code that makes the failing test pass. MUST NOT add behavior beyond what the current test requires.

8. **Verify pass**:
   Run the test to confirm it passes. If it fails, fix the implementation -- not the test -- unless the test itself has a bug.

9. **Refactor** (optional for new behavior; forbidden during bug fix):
   **Never refactor while RED.** Get to GREEN first. For new behavior: SHOULD refactor if duplication reaches 3 or more repetitions, naming has drifted, or structure obscures intent. All tests MUST still pass after each refactor step. For bug fixes: do NOT refactor during this cycle -- open a separate Apply-mode pass afterward if cleanup is warranted.

10. **Per-cycle checklist**:
    Before moving to the next test case (or reporting done), verify:
    - [ ] Test describes behavior, not implementation
    - [ ] Test uses public interface only
    - [ ] Test would survive internal refactor
    - [ ] Code is minimal for this test
    - [ ] No speculative features added

    See references/test-examples.md for good-vs-bad test examples, references/interface-design.md for testability principles, and references/mocking.md for when mocking is appropriate.

11. **Repeat for remaining test cases** (new-behavior sub-case):
    Cycle through steps 5–10 for each planned test case. Adjust granularity by complexity -- simple cases MAY be batched, complex cases SHOULD get individual red-green-refactor cycles.

    Consider references/refactor-smells.md and references/deep-modules.md when refactoring -- if the code reveals a shallow module or coupling problem that's beyond the scope of this cycle's refactor, note it for a later `code architect` invocation.

12. **Report results**:
    MUST report to the user:
    - Number of tests written (1 for bug fixes)
    - Behaviors covered
    - Any deferred edge cases, test scenarios, or architectural smells noted during development
