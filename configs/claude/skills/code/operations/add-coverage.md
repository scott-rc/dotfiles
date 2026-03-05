# Add Coverage

Write characterization tests for existing untested code — documenting its current behavior as a test suite.

## Instructions

1. **Load coding guidelines**: Follow references/load-guidelines.md.

2. **Identify target code**:
   Determine which module, function, or class to cover. If the user hasn't specified what to cover, ask them before proceeding.

3. **Map coverage gaps**:
   Scan the project for test files matching the target code (common patterns: `*.test.*`, `*.spec.*`, `*_test.*`, `test_*.*`, `__tests__/`). For each target file, locate its test file and read both. Identify exported/public function signatures, which functions have test coverage, and which lack it.

4. **Analyze code behavior**:
   Spawn a Task subagent (type: Explore) with the target files and coverage map. The subagent MUST read the target code and its callers, then return a structured test plan: one entry per test case with input, expected behavior, and the branch or edge case it covers. Present the plan to the user before writing tests.

5. **Write tests incrementally**:
   Write tests in small batches. Every test MUST pass — these tests document what the code currently does, not what it should do. If a test reveals behavior that looks like a bug, write the test to match current behavior and note the suspected bug; do not fix it in this operation.

6. **Verify no regressions**:
   Run the full test suite to confirm no regressions.

7. **Report results**:
   MUST report to the user:
   - Number of tests added
   - Behaviors documented
   - Any potential bugs discovered (with specifics)
