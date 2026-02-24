# Add Coverage

Write characterization tests for existing untested code — documenting its current behavior as a test suite.

## Instructions

1. **Load preferences and resolve test environment**:
   MUST read [test-environment.md](test-environment.md), [general-guidelines.md](general-guidelines.md), [testing-guidelines.md](testing-guidelines.md), and the language-specific guidelines if available. Apply these throughout.

3. **Identify target code**:
   Determine which module, function, or class to cover. If the user hasn't specified, spawn a Task subagent (type: Explore, model: haiku) to scan the codebase for untested code -- it should check for files without corresponding test files and public functions without test coverage. Present 1-3 untested candidates as AskUserQuestion options.

4. **Plan characterization tests**:
   Spawn a Task subagent (type: Explore, model: sonnet) to read the target code and its callers, then return a structured analysis: function signatures, branches, edge cases, error paths, and boundary conditions. Use the analysis to draft a list of test cases that document current behavior. Present the plan to the user before writing tests.

5. **Write tests incrementally**:
   Write tests in small batches. Every test MUST pass — these tests document what the code currently does, not what it should do.
   - If a test reveals behavior that looks like a bug, note it but write the test to match current behavior.
   - Report suspected bugs to the user as you go — do not fix them in this operation.

6. **Report results**:
   MUST report to the user:
   - Number of tests added
   - Behaviors documented
   - Any potential bugs discovered (with specifics)
