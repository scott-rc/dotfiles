# New Feature

Build a feature through the red-green-refactor cycle: write a failing test, make it pass with minimal code, then clean up.

## Instructions

1. **Load preferences and resolve test environment**:
   MUST read [test-environment.md](test-environment.md), [general-guidelines.md](general-guidelines.md), [testing-guidelines.md](testing-guidelines.md), and the language-specific guidelines if available. Apply these throughout.

3. **Understand the feature**:
   Clarify what the feature does, its inputs, outputs, and edge cases. If the user's request is vague, ask focused questions before proceeding.

4. **Plan test cases**:
   Draft 2–5 test cases covering the core behavior and important edge cases. Apply the special cases checklist from [testing-guidelines.md](testing-guidelines.md) to catch boundary conditions. Present the list to the user for confirmation before writing any code. Adjust based on their feedback.

5. **Red — write a failing test**:
   Write the first test case. It MUST assert the expected behavior of the feature. The test SHOULD fail because the implementation doesn't exist yet.

6. **Verify failure**:
   MUST run the first test case to verify the harness works. MAY skip for subsequent cases if the runner and assertion pattern are established.

7. **Green — write minimal implementation**:
   Write the simplest code that makes the failing test pass. Do not add behavior beyond what the current test requires.

8. **Verify pass**:
   Run the test to confirm it passes. If it fails, fix the implementation — not the test — unless the test itself has a bug.

9. **Refactor** (optional):
   SHOULD refactor if duplication exceeds 2 repetitions, naming has drifted, or structure obscures intent. All tests MUST still pass after refactoring.

10. **Repeat for remaining test cases**:
    Cycle through steps 5–9 for each planned test case. Adjust granularity by complexity — simple cases MAY be batched, complex cases SHOULD get individual red-green-refactor cycles.

11. **Report results**:
    MUST report to the user:
    - Number of tests written
    - Behaviors covered
    - Any deferred edge cases or test scenarios noted during development
