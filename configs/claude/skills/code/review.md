# Review Code

Evaluate code for test gaps, idiomaticity, simplification opportunities, and other issues — producing a structured findings report.

## Instructions

1. **Identify review scope**:
   Determine what code to review. If the user specifies files, functions, or a diff, use that. If unspecified, ask what they want reviewed.

2. **Read general preferences**: MUST load [general-guidelines.md](general-guidelines.md). These apply to all languages.

3. **Read language-specific preferences** (if available):
   - **TypeScript**: [typescript-guidelines.md](typescript-guidelines.md)
   - **Go**: [go-guidelines.md](go-guidelines.md)
   - **Bash / Fish**: [shell-guidelines.md](shell-guidelines.md)

   If no file exists for the target language, use only the general guidelines.

4. **Study project context**:
   Read surrounding code to understand project conventions — naming patterns, error handling style, abstraction level, test patterns. The review MUST judge code against its own project's standards, not abstract ideals.

5. **Run review checklist**:
   Evaluate every item below. Track findings with severity:
   - **issue** — likely bug, missing error handling at a boundary, or correctness problem
   - **suggestion** — improvement that makes code clearer, simpler, or more maintainable
   - **nit** — minor style or preference item

   ### Test Coverage
   - Are exported/public functions covered by tests?
   - Are important edge cases tested (empty inputs, boundary values, error paths)?
   - Do tests assert behavior and outcomes, not implementation details?
   - Are there untested error handling paths at system boundaries?
   - If no tests exist for the code under review, flag it — but distinguish between code that needs tests (business logic, parsers, state machines) and code where tests add little value (thin wrappers, config, glue code).

   ### Idiomaticity
   - Does the code follow the loaded coding guidelines?
   - Does the code match surrounding project conventions (naming, patterns, structure)?
   - Are language-specific idioms used where appropriate (e.g., pattern matching instead of if-chains in Rust, guard clauses instead of nested ifs)?
   - Are framework/library APIs used as intended, not fought against?

   ### Simplification
   - Can any function be split because it does multiple unrelated things?
   - Is there duplicated logic that has appeared 3+ times and should be extracted?
   - Are there premature abstractions — wrappers, helpers, or indirection layers that serve only one call site?
   - Can nested conditionals be flattened with guard clauses or early returns?
   - Is there dead code (unreachable branches, unused variables, commented-out code)?
   - Are there overly defensive checks for conditions that cannot occur internally?

   ### Correctness and Robustness
   - Is error handling present at system boundaries (user input, API responses, file I/O)?
   - Are there race conditions, missing null checks on external data, or unhandled promise rejections?
   - Are resource cleanup paths correct (streams closed, connections released, listeners removed)?

   ### Naming and Clarity
   - Do names communicate purpose at the call site?
   - Are there misleading names (e.g., a function named `get*` that mutates state)?
   - Are "why" comments present for non-obvious logic? Are there comments that just restate the code?

6. **Report findings**:
   MUST present findings grouped by severity (issues first, then suggestions, then nits). Each finding MUST include:
   - File and location
   - What the problem is (one sentence)
   - A concrete fix or recommendation

   If no findings, say so — do not manufacture issues.
