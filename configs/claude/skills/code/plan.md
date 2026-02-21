# Plan

Design an implementation plan for a coding task, defaulting to TDD structure when the task involves testable behavior.

## Instructions

1. **Load coding preferences**:
   MUST read [general-guidelines.md](general-guidelines.md) and the language-specific guidelines if available ([typescript-guidelines.md](typescript-guidelines.md), [go-guidelines.md](go-guidelines.md), [shell-guidelines.md](shell-guidelines.md)).

2. **Explore the codebase**:
   Read relevant source files, existing tests, and project structure. Understand the patterns, conventions, and test infrastructure already in place.

3. **Classify the task**:
   Determine whether the implementation involves testable behavior:
   - **Testable** — new functionality, bug fix, behavioral change, API addition, logic that produces observable outputs. Use TDD plan structure.
   - **Not testable** — refactoring under existing coverage, config changes, glue code, shell scripts, one-liners, purely structural moves. Use Apply plan structure.

   If ambiguous, default to TDD. The user can opt out.

4. **Design the plan**:

   **TDD plan structure** — organize implementation steps around the red-green-refactor cycle:
   - Identify test cases (2-5) covering core behavior and key edge cases
   - Order steps so each test case drives one increment of implementation
   - Each step: write failing test, write minimal implementation to pass, refactor if needed
   - Call out the test runner and file placement conventions (reference [test-environment.md](test-environment.md) if not yet resolved)

   **Apply plan structure** — organize around the code changes directly:
   - List files to modify and what changes in each
   - Identify verification steps (existing tests to run, manual checks)

5. **Present the plan**:
   Write the plan to the plan file. For TDD plans, MUST include the planned test cases so the user can evaluate coverage before implementation begins.
