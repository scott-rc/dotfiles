# Plan

Decompose a coding task into ordered chunks, defaulting to TDD structure for testable behavior.

## Instructions

1. **Load coding preferences**:
   MUST read [general-guidelines.md](general-guidelines.md) and the language-specific guidelines if available ([typescript-guidelines.md](typescript-guidelines.md), [go-guidelines.md](go-guidelines.md), [shell-guidelines.md](shell-guidelines.md)).
   Also read [test-environment.md](test-environment.md) to understand test runner and file placement conventions.
   Note the build commands, test commands, test file patterns, and framework details -- you will need these for step 2.

2. **Invoke the compose skill**:
   MUST use the Skill tool to invoke the compose skill (`skill: "compose", args: "plan this task"`). Do NOT attempt ad-hoc planning or skip this step. The compose skill's plan-task workflow is the only valid way to produce a plan.

   This loads the full plan-task workflow with its templates and references. The compose skill will run the structured workflow: interview, explore, confirm, decompose, write chunks, write master plan, validate, deliver.

   While executing the compose plan-task workflow, apply these coding-specific constraints:
   - During codebase exploration, identify test infrastructure, existing test patterns, and the test runner
   - When decomposing into chunks, default to TDD structure for chunks that add testable behavior: each such chunk's implementation steps should follow red-green-refactor (write failing test, implement to pass, refactor)
   - Chunks that are pure refactoring, config, or glue code do not need TDD structure
   - Include build and test commands discovered from test-environment.md in the chunk context
