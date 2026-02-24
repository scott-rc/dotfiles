# Fix Bug

Write a regression test that reproduces the bug, then fix it — ensuring the bug cannot silently return.

## Instructions

1. **Load preferences and resolve test environment**:
   MUST read [test-environment.md](test-environment.md), [general-guidelines.md](general-guidelines.md), [testing-guidelines.md](testing-guidelines.md), and the language-specific guidelines if available. Apply these throughout.

3. **Understand the bug**:
   Identify the expected behavior and the actual (broken) behavior. Read the relevant source code. If the bug is unclear, ask the user to describe the reproduction steps.

4. **Write regression test**:
   Write a test that asserts the correct (expected) behavior. This test MUST fail against the current buggy code — it captures exactly what's broken.

5. **Verify failure**:
   Run the test to confirm it fails for the right reason. If it passes, the test isn't capturing the bug — revisit step 4.

6. **Fix the bug**:
   Make the minimal change to fix the bug. Do not refactor or improve surrounding code — keep the diff focused on the fix.

7. **Verify the fix**:
   Run the regression test to confirm it passes. Then run the full test suite (or the relevant subset) to confirm no other tests broke.

8. **Report results**:
   MUST report to the user:
   - What the regression test covers
   - What changed in the fix
   - Whether any other tests were affected
