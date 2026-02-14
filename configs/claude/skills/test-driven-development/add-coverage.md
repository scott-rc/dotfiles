# Add Coverage

Write characterization tests for existing untested code — documenting its current behavior as a test suite.

## Instructions

1. **Resolve test environment**:
   MUST read [test-environment.md](test-environment.md) and follow its detection steps to determine the test runner, file placement, and naming convention.

2. **Identify target code**:
   Determine which module, function, or class to cover. If the user hasn't specified, ask what code they want tests for.

3. **Plan characterization tests**:
   Read the target code and draft a list of test cases that document its current behavior — happy paths, edge cases, error handling, and boundary conditions. Present the plan to the user before writing tests.

4. **Write tests incrementally**:
   Write tests in small batches. Every test MUST pass — these tests document what the code currently does, not what it should do.
   - If a test reveals behavior that looks like a bug, note it but write the test to match current behavior.
   - Report suspected bugs to the user as you go — do not fix them in this operation.

5. **Report results**:
   MUST report to the user:
   - Number of tests added
   - Behaviors documented
   - Any potential bugs discovered (with specifics)

SHOULD apply coding style preferences from the `writing-code` skill when writing test code.
