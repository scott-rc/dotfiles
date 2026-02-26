# Fix Bug

Write a regression test that reproduces the bug, then fix it — ensuring the bug cannot silently return.

## Instructions

1. **Load coding guidelines**: Follow [load-guidelines.md](load-guidelines.md).

2. **Understand the bug**:
   MUST identify the expected behavior and the actual (broken) behavior. Read the relevant source code. If the bug is unclear, ask the user to describe the reproduction steps.

3. **Write regression test**:
   Write a test that asserts the correct (expected) behavior. This test MUST fail against the current buggy code — it captures exactly what's broken.

4. **Verify failure**:
   Run the test to confirm it fails for the right reason. If it passes, the test isn't capturing the bug — revisit step 3.

5. **Fix the bug**:
   Make the minimal change to fix the bug. MUST NOT refactor or improve surrounding code — keep the diff focused on the fix.

6. **Verify the fix**:
   Run the regression test to confirm it passes. Then run the full test suite (or the relevant subset) to confirm no other tests broke.

7. **Report results**:
   MUST report to the user:
   - What the regression test covers
   - What changed in the fix
   - Whether any other tests were affected
