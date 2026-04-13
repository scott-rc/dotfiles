# Write Code

Load coding guidelines, write code, and verify it — supports three modes: Apply (no TDD), Feature (red-green-refactor), and Fix (regression-first bug fix).

## Instructions

### Shared Setup

1. **Load coding guidelines**: Follow references/load-guidelines.md.

2. **Identify the mode**:
   - **Apply** — no tests needed: refactoring under existing coverage, config, glue code, one-liners, or user opts out of TDD
   - **Feature** — new behavior that warrants test-first development
   - **Fix** — a bug to correct; requires a regression test before the fix

   **Key test**: "does this change what the system does when it runs?" If yes → Feature mode, even if the underlying API already exists.

   Common misclassifications — these are Feature mode, not Apply:
   - "ensure X uses URL params for state" — adds server-side param reading + client-side sync (new behavior)
   - "make the API return pagination metadata" — adds response fields (new behavior)
   - "add caching to the handler" — adds cache logic (new behavior)
   - "sync state between X and Y" — adds synchronization that didn't exist (new behavior)

   If the mode is ambiguous, ask the user before proceeding.

---

## Apply Mode

Write code without TDD — for refactoring, config, glue, scripting, or when the user explicitly opts out of tests.

3. **Scope and write the code**: For broad maintainability or refactor requests, MUST identify the primary seam first, state the expected touched files, and call out unrelated working-tree changes before editing. Prefer the smallest clear edit sequence over whole-file rewrites; rewrite an entire file only when the structural simplification clearly outweighs diff churn. Apply the loaded preferences. When they conflict with existing project conventions (linter config, formatter, existing patterns), SHOULD follow project conventions.

4. **Self-check**: MUST verify code follows the loaded guidelines, paying special attention to naming, comments, nesting, error handling, and abstractions. SHOULD run the relevant formatter, linter, or static analysis used by the project when the change touches code those tools govern, even for refactors. For shell scripts, MUST run `shellcheck` and fix all warnings. If any violation is found, fix it and re-check. Iterate up to 3 times. If violations persist after 3 iterations, report remaining issues to the user.

5. **Run existing tests**: If the project has a test suite, run it (or the relevant subset) to confirm no regressions.

6. **Report results**: Present the code to the user with a summary of which preferences were applied, any conflicts resolved, touched files, and test suite status (pass/fail, number of tests run).

---

## Feature Mode

Build a feature through the red-green-refactor cycle: write a failing test, make it pass with minimal code, then clean up.

3. **Understand the feature**:
   Clarify what the feature does, its inputs, outputs, and edge cases. If the user's request is vague, ask focused questions before proceeding.

4. **Plan test cases**:
   Draft 2–5 test cases covering the core behavior and important edge cases. Apply the special cases checklist from references/testing-guidelines.md to catch boundary conditions. Present the list to the user for confirmation before writing any code. Adjust based on their feedback.

5. **Red — write a failing test**:
   Write the first test case. It MUST assert the expected behavior of the feature. The test SHOULD fail because the implementation doesn't exist yet.

6. **Verify failure**:
   MUST run the first test case to verify the harness works. MAY skip for subsequent cases if the runner and assertion pattern are established.

7. **Green — write minimal implementation**:
   Write the simplest code that makes the failing test pass. MUST NOT add behavior beyond what the current test requires.

8. **Verify pass**:
   Run the test to confirm it passes. If it fails, fix the implementation — not the test — unless the test itself has a bug.

9. **Refactor** (optional):
   SHOULD refactor if duplication reaches 3 or more repetitions, naming has drifted, or structure obscures intent. All tests MUST still pass after refactoring.

10. **Repeat for remaining test cases**:
    Cycle through steps 5–9 for each planned test case. Adjust granularity by complexity — simple cases MAY be batched, complex cases SHOULD get individual red-green-refactor cycles.

11. **Report results**:
    MUST report to the user:
    - Number of tests written
    - Behaviors covered
    - Any deferred edge cases or test scenarios noted during development

---

## Fix Mode

Write a regression test that reproduces the bug, then fix it — ensuring the bug cannot silently return.

3. **Understand the bug**:
   MUST identify the expected behavior and the actual (broken) behavior. Read the relevant source code. If the bug is unclear, ask the user to describe the reproduction steps.

4. **Write regression test**:
   Write a test that asserts the correct (expected) behavior. This test MUST fail against the current buggy code — it captures exactly what's broken.

5. **Verify failure**:
   Run the test to confirm it fails for the right reason. If it passes, the test isn't capturing the bug — revisit step 4.

6. **Fix the bug**:
   Make the minimal change to fix the bug. MUST NOT refactor or improve surrounding code — keep the diff focused on the fix.

7. **Verify the fix**:
   Run the regression test to confirm it passes. Then run the full test suite (or the relevant subset) to confirm no other tests broke.

8. **Report results**:
   MUST report to the user:
   - What the regression test covers
   - What changed in the fix
   - Whether any other tests were affected
