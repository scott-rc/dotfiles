# Plan

Delegates to the compose skill. This operation does NOT produce a plan directly.

## Instructions

1. **Invoke compose immediately**:
   MUST use the Skill tool: `skill: "compose", args: "plan this task"`.
   Do NOT read files, explore the codebase, enter plan mode, or start implementing first.

## Coding Constraints

Apply these during the compose workflow's codebase exploration and decomposition:

- Read [general-guidelines.md](general-guidelines.md) and applicable language-specific guidelines ([typescript-guidelines.md](typescript-guidelines.md), [go-guidelines.md](go-guidelines.md), [shell-guidelines.md](shell-guidelines.md))
- Read [test-environment.md](test-environment.md) for test runner, build commands, and file conventions
- Default to TDD structure (red-green-refactor) for chunks adding testable behavior
- Chunks that are pure refactoring, config, or glue code do not need TDD structure
- Include build and test commands in chunk context

## Anti-patterns

- Reading files before invoking compose — the compose workflow has its own exploration step
- Using `EnterPlanMode` — this operation delegates via the Skill tool, not plan mode
- Writing a plan directly — output MUST be chunk files from compose, not prose
