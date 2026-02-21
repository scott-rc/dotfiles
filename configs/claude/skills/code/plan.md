# Plan

Decompose a coding task into ordered chunks, defaulting to TDD structure for testable behavior.

This operation delegates to the compose skill's plan-task workflow. It does NOT produce a plan directly.

## Instructions

1. **Load coding context** (read only — do NOT begin planning):
   MUST read [general-guidelines.md](general-guidelines.md) and the language-specific guidelines if available ([typescript-guidelines.md](typescript-guidelines.md), [go-guidelines.md](go-guidelines.md), [shell-guidelines.md](shell-guidelines.md)).
   Also read [test-environment.md](test-environment.md) to understand test runner and file placement conventions.
   Note the build commands, test commands, test file patterns, and framework details — you will need these during the compose workflow.

2. **Delegate to compose** (MANDATORY):
   MUST use the Skill tool NOW to invoke the compose skill: `skill: "compose", args: "plan this task"`. Do NOT write a plan, enter plan mode, or start implementing. The compose skill's plan-task workflow is the ONLY valid way to produce a plan — it writes chunk files and a master plan, not prose.

   During the compose workflow, apply these coding-specific constraints:
   - During codebase exploration, identify test infrastructure, existing test patterns, and the test runner
   - Default to TDD structure (red-green-refactor) for chunks that add testable behavior
   - Chunks that are pure refactoring, config, or glue code do not need TDD structure
   - Include build and test commands from test-environment.md in chunk context

## Anti-patterns

- Using `EnterPlanMode` instead of invoking compose via the Skill tool
- Writing a plan directly — the output MUST be chunk files, not a text plan
- Starting implementation before the plan is approved
- Skipping the Skill tool invocation
