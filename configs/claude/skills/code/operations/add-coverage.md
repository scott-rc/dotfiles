# Add Coverage Operation

Write characterization tests for existing untested code — documenting its current behavior as a test suite.

## Instructions

1. **Load coding guidelines**: Follow references/load-guidelines.md.

2. **Identify target code**:
   Determine which module, function, or class to cover. If the user hasn't specified what to cover, ask them before proceeding.

3. **Map coverage gaps**:
   Spawn the `test-discoverer` agent with the target files to get a coverage map: function signatures, covered/uncovered functions, and notable gaps.

4. **Analyze code behavior**:
   Spawn a Task subagent (type: Explore) to read the target code and its callers, then return a structured analysis: branches, edge cases, error paths, and boundary conditions. Use both the coverage map and the code analysis to draft a list of test cases that document current behavior. Present the plan to the user before writing tests.

5. **Write tests incrementally**:
   Write tests in small batches. Every test MUST pass — these tests document what the code currently does, not what it should do. If a test reveals behavior that looks like a bug, write the test to match current behavior and note the suspected bug; do not fix it in this operation.

6. **Report results**:
   MUST report to the user:
   - Number of tests added
   - Behaviors documented
   - Any potential bugs discovered (with specifics)
