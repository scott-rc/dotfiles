# Plan

Decompose a coding task into ordered chunks, defaulting to TDD structure for testable behavior.

## Instructions

1. **Load coding preferences**:
   MUST read [general-guidelines.md](general-guidelines.md) and the language-specific guidelines if available ([typescript-guidelines.md](typescript-guidelines.md), [go-guidelines.md](go-guidelines.md), [shell-guidelines.md](shell-guidelines.md)).
   Also read [test-environment.md](test-environment.md) to understand test runner and file placement conventions.

2. **Run the Plan Task operation**:
   MUST read and follow [../compose/plan-task.md](../compose/plan-task.md) for the full decomposition workflow (interview, explore, confirm, decompose, write chunks, write master plan, validate, deliver).

   When executing that workflow, apply these coding-specific constraints:
   - During codebase exploration, identify test infrastructure, existing test patterns, and the test runner
   - When decomposing into chunks, default to TDD structure for chunks that add testable behavior: each such chunk's implementation steps should follow red-green-refactor (write failing test, implement to pass, refactor)
   - Chunks that are pure refactoring, config, or glue code do not need TDD structure
   - Include build and test commands discovered from [test-environment.md](test-environment.md) in the chunk context
   - The plan-task operation links to reference files in the compose skill directory. MUST read any reference file it links to before proceeding (especially [../compose/plan-template.md](../compose/plan-template.md) — templates for chunk files, master plan, and orchestrator prompt).
