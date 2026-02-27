# Mutation Testing

Evaluate test suite quality by introducing mutations into source code and checking whether tests catch them, then write tests to kill any survivors.

## Instructions

1. **Load coding guidelines**: Follow references/load-guidelines.md.

2. **Identify target code and tests**:
   If the user specified target files, spawn the `test-discoverer` agent in targeted mode with the specified files. Otherwise, spawn the `test-discoverer` agent in auto-discover mode to identify candidates.
   Use the agent's coverage map to locate source files, corresponding test files, function signatures, and coverage status.
   If no tests exist for the target code, stop and suggest Add Coverage instead.

3. **Plan mutations**:
   Spawn a Task subagent (type: Explore) to read the target code and propose 5-15 mutations. Each mutation MUST:
   - Be a small, single-point change (one operator, one condition, one return value)
   - Target meaningful behavior (not comments, whitespace, or unreachable code)
   - Be categorized by type: boundary (off-by-one, `<` vs `<=`), negation (`!` flip), return value (wrong constant, null/empty), operator (`+` vs `-`, `&&` vs `||`), deletion (remove a call, skip a branch), or argument swap

   The subagent MUST return a numbered list with: mutation type, file path, line number, original code, mutated code, and which behavior it tests.

   Present the mutation plan to the user via AskUserQuestion with options: "Proceed with all mutations", "Adjust count", "Pick specific mutations". If the user wants changes, adjust and re-present.

4. **Execute mutations via mutation-executor agent**:
   Spawn the `mutation-executor` agent with the planned mutations list (each with index, type, file path, original code, mutated code, and description) and the test command.

   The agent handles the full apply-test-record-revert cycle for each mutation and returns a results table with killed/survived outcomes and a mutation score.

5. **Analyze surviving mutants**:
   Using the mutation-executor's results, for each surviving mutant identify what behavior is unguarded:
   - Which function or branch does the mutation affect?
   - What input would distinguish the original from the mutant?
   - What assertion is missing from the existing tests?

   If all mutants were killed, report the clean result and stop -- the test suite is solid for this code.

6. **Write tests to kill survivors**:
   Apply references/testing-guidelines.md (loaded in step 1). For each surviving mutant:
   - Write a test case that would fail against the mutant but pass against the original
   - Run the new test to confirm it passes on the original code
   - Apply the mutation again and run the new test to confirm it fails (the mutant is now killed)
   - Revert the mutation

   If a survivor requires integration-level verification to kill, note it in the report rather than forcing a unit test.

7. **Report results**:
   MUST report to the user:
   - Mutation score: killed / total (e.g., "12/15 — 80%")
   - Score after new tests: killed / total (e.g., "15/15 — 100%")
   - Per-mutant results: type, location, outcome, and the test that killed it (or why it was left alive)
   - Gaps found: plain-language summary of what the test suite was missing
   - Test suite status: confirm all tests pass on the unmodified code
