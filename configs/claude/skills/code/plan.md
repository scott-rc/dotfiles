# Plan

Decompose a coding task into ordered chunks with TDD structure for testable behavior.

**Delegates to the compose skill -- do NOT use EnterPlanMode.**

## Instructions

Invoke compose immediately: `skill: "compose", args: "plan this task"`. Do NOT read files, explore the codebase, or start implementing first.

During the compose workflow, apply these coding constraints:
- Read [general-guidelines.md](general-guidelines.md) and applicable language-specific guidelines
- Read [test-environment.md](test-environment.md) for test runner, build commands, and file conventions
- Default to TDD structure (red-green-refactor) for chunks adding testable behavior
- Chunks that are pure refactoring, config, or glue code do not need TDD structure
