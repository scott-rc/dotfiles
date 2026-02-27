# Plan Operation

Decompose a coding task into ordered chunks with TDD structure for testable behavior.

## Instructions

1. **Invoke compose**: Delegate immediately to the compose skill with `skill: "compose", args: "plan this task"`. Do NOT read files, explore the codebase, or start implementing first.

2. **Pass coding constraints**: Provide these constraints as context to compose:
   - Default to TDD structure (red-green-refactor) for chunks that add testable behavior
   - Chunks that are pure refactoring, config, or glue code do not need TDD structure
   - Guidelines: references/general-guidelines.md, applicable language-specific guidelines, and references/test-environment.md for runner and file conventions
