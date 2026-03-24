# Test

Improve test coverage or evaluate test suite quality — supports two modes: Coverage (characterization tests for untested code) and Mutate (mutation testing to find and kill survivors).

## Instructions

### Shared Setup

1. **Load coding guidelines**: Follow references/load-guidelines.md.

2. **Identify the mode**:
   - **Coverage** — write characterization tests for existing untested code
   - **Mutate** — run mutation testing to evaluate test quality, then write tests to kill survivors

   If the mode is ambiguous, ask the user before proceeding.

3. **Locate test files**:
   Scan the project for test files matching the target code (common patterns: `*.test.*`, `*.spec.*`, `*_test.*`, `test_*.*`, `__tests__/`). For each target file, locate its test file and read both. Identify exported/public function signatures, which functions have test coverage, and which lack it.

---

## Coverage Mode

Write characterization tests for existing untested code — documenting its current behavior as a test suite.

4. **Identify target code**:
   Determine which module, function, or class to cover. If the user hasn't specified what to cover, ask them before proceeding.

5. **Analyze code behavior**:
   Spawn a Task subagent (type: Explore) with the target files and coverage map from step 3. The subagent MUST read the target code and its callers, then return a structured test plan: one entry per test case with input, expected behavior, and the branch or edge case it covers. Present the plan to the user before writing tests.

6. **Write tests incrementally**:
   Write tests in small batches. Every test MUST pass — these tests document what the code currently does, not what it should do. If a test reveals behavior that looks like a bug, write the test to match current behavior and note the suspected bug; do not fix it in this operation.

7. **Verify no regressions**:
   Run the full test suite to confirm no regressions.

8. **Report results**:
   MUST report to the user:
   - Number of tests added
   - Behaviors documented
   - Any potential bugs discovered (with specifics)

---

## Mutate Mode

Evaluate test suite quality by introducing mutations into source code and checking whether tests catch them, then write tests to kill any survivors.

4. **Check test coverage exists**:
   Using the test file map from step 3, confirm tests exist for the target code. If no tests exist, stop and suggest Coverage mode instead.

5. **Plan mutations**:
   Spawn a Task subagent (type: Explore) to read the target code and propose 5-15 mutations. Each mutation MUST be a small, single-point change targeting meaningful behavior (not comments, whitespace, or unreachable code).

   The subagent MUST return a numbered list with: mutation type, file path, line number, original code, mutated code, and which behavior it tests.

   Present the mutation plan to the user via AskUserQuestion with options: "Proceed with all mutations", "Adjust count", "Pick specific mutations".
   - **Proceed** → continue to step 6 with all mutations
   - **Adjust count** → re-run the Explore subagent with the new count
   - **Pick specific** → filter the list to user-selected mutations, then continue to step 6

6. **Execute mutations**:
   Run a baseline test (`<test_command>`) to confirm all tests pass. If tests already fail, stop -- fix them first.

   For each mutation in order:
   a. **Apply**: Edit the source file with the mutation's original → mutated code
   b. **Run tests**: Execute `<test_command>`, capture exit code
   c. **Record**: non-zero exit with test failures → **killed**; non-zero with compile error → **killed (compile)**; zero (tests pass) → **survived**
   d. **Revert**: Edit back to original. MUST revert before the next mutation. If revert fails, stop.

   MUST NOT stack mutations. MUST NOT modify test files. Skip mutations where the edit fails (original string not found) and note as **skipped**.

   Run a final baseline test to confirm the code is back to its original state.

7. **Analyze surviving mutants**:
   Using the results from step 6, for each surviving mutant identify what behavior is unguarded:
   - Which function or branch does the mutation affect?
   - What input would distinguish the original from the mutant?
   - What assertion is missing from the existing tests?

   If all mutants were killed, report the clean result and stop — the test suite is solid for this code.

8. **Write tests to kill survivors**:
   Apply references/testing-guidelines.md. For each surviving mutant:
   - Write a test case that would fail against the mutant but pass against the original
   - Run the new test to confirm it passes on the original code
   - Apply the mutation again and run the new test to confirm it fails (the mutant is now killed)
   - Revert the mutation

   If a survivor requires integration-level verification to kill, note it in the report rather than forcing a unit test.

9. **Report results**:
   MUST report to the user:
   - Mutation score: killed / total (e.g., "12/15 — 80%")
   - Score after new tests: killed / total (e.g., "15/15 — 100%")
   - Per-mutant results: type, location, outcome, and the test that killed it (or why it was left alive)
   - Gaps found: plain-language summary of what the test suite was missing
   - Test suite status: confirm all tests pass on the unmodified code
