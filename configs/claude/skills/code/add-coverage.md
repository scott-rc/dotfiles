# Add Coverage

Write characterization tests for existing untested code — documenting its current behavior as a test suite.

## Instructions

1. **Load coding guidelines**: Follow [load-guidelines.md](load-guidelines.md).

2. **Identify target code**:
   Determine which module, function, or class to cover. If the user hasn't specified, spawn the `test-discoverer` agent in auto-discover mode. Present 1-3 untested candidates from the agent's results as AskUserQuestion options.

3. **Plan characterization tests**:
   If the test-discoverer was not already run in targeted mode for the chosen file, spawn it now with the target files to get a detailed coverage map (function signatures, covered/uncovered functions, notable gaps).
   Spawn a Task subagent (type: Explore) to read the target code and its callers, then return a structured analysis: branches, edge cases, error paths, and boundary conditions. Use both the coverage map and the code analysis to draft a list of test cases that document current behavior. Present the plan to the user before writing tests.

4. **Write tests incrementally**:
   Write tests in small batches. Every test MUST pass — these tests document what the code currently does, not what it should do.
   - If a test reveals behavior that looks like a bug, note it but write the test to match current behavior.
   - Report suspected bugs to the user as you go — do not fix them in this operation.

5. **Report results**:
   MUST report to the user:
   - Number of tests added
   - Behaviors documented
   - Any potential bugs discovered (with specifics)
