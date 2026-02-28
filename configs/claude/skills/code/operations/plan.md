# Plan

Decompose a coding task into ordered chunks with TDD structure for testable behavior.

## Instructions

1. **Invoke compose**: Delegate immediately to the compose skill with `skill: "compose", args: "$ARGUMENTS"`. Do NOT read files, explore the codebase, or start implementing first.

2. **Pass coding constraints**: Provide these constraints as context to compose:
   - Default to TDD structure (red-green-refactor) for chunks that add testable behavior
   - Chunks that are pure refactoring, config, or glue code do not need TDD structure
   - Use the project's detected test runner and file placement conventions when structuring test chunks
   - Apply general coding guidelines (naming, control flow, error handling) and any applicable language-specific conventions

3. **Report**: Confirm the plan was delivered by compose and present any summary to the user.
