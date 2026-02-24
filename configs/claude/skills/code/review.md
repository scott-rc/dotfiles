# Review Code

Evaluate code for test gaps, idiomaticity, simplification opportunities, and other issues — producing a structured findings report.

## Instructions

1. **Identify review scope**:
   Determine what code to review. If the user specifies files, functions, or a diff, use that. If unspecified, ask what they want reviewed.

2. **Assess scope size**:
   Count changed files and lines (`git diff --stat` or file list). Apply this heuristic:
   - **Small scope**: ≤8 changed files AND ≤500 changed lines → follow steps 3–6
   - **Large scope**: >8 changed files OR >500 changed lines → follow steps 7–13

   Present the assessment via AskUserQuestion with options: "Quick review (single-pass)", "Thorough review (subagent decomposition)". The user can also force the thorough path by saying "thorough review" or "deep review".

### Small Scope (Single-Pass Review)

3. **Load guidelines**: MUST load [general-guidelines.md](general-guidelines.md) and [testing-guidelines.md](testing-guidelines.md). If a language-specific file exists for the target language ([typescript-guidelines.md](typescript-guidelines.md), [go-guidelines.md](go-guidelines.md), [shell-guidelines.md](shell-guidelines.md)), load it too.

4. **Study project context**:
   Spawn a Task subagent (type: Explore, model: haiku) to read surrounding code and identify project conventions -- naming patterns, error handling style, abstraction level, test patterns. The subagent should return a concise summary of conventions found. The review MUST judge code against its own project's standards, not abstract ideals.

5. **Run review checklist**:
   Evaluate every item in the [Review Checklist](#review-checklist) below. Track findings with severity:
   - **issue** — likely bug, missing error handling at a boundary, or correctness problem
   - **suggestion** — improvement that makes code clearer, simpler, or more maintainable
   - **nit** — minor style or preference item

6. **Report findings**:
   MUST present findings grouped by severity (issues first, then suggestions, then nits). Each finding MUST include:
   - File and location
   - What the problem is (one sentence)
   - A concrete fix or recommendation

   If no findings, say so — do not manufacture issues.

### Large Scope (Subagent Decomposition)

7. **Load guidelines**: MUST load [general-guidelines.md](general-guidelines.md). If language-specific files exist for the target languages ([typescript-guidelines.md](typescript-guidelines.md), [go-guidelines.md](go-guidelines.md), [shell-guidelines.md](shell-guidelines.md)), load them too.

8. **Decompose into review scopes**:
   Map all changes (`git diff --stat` or file list). Decompose into 2–5 independent review scopes. Each scope gets:
   - **Name** — short descriptive label
   - **Files** — concrete file list (non-overlapping across scopes)
   - **Focus** — what to pay attention to in this scope
   - **Criteria** — scope-specific review criteria beyond the standard checklist

   Present the scopes to the user for approval before proceeding.

9. **Load review template**: Read [review-template.md](review-template.md) for the subagent prompt template. Use this template when spawning each review subagent below.

10. **Spawn review subagents**:
    Launch parallel Task subagents (type: code-reviewer), one per scope. Fill in the template with:
    - The scope's name, file list, and focus
    - Paths to the guidelines files loaded in step 7
    - Project context (repo root, conventions observed)
    - The full [Review Checklist](#review-checklist) below (paste all five sections into the template's checklist placeholder)
    - Scope-specific criteria

11. **Consolidate findings**:
    Merge all subagent findings. Deduplicate by file + location. Group by severity (issues → suggestions → nits).

12. **Report findings**:
    Same format as step 6 — grouped by severity, each with file/location, problem description, and concrete fix.

13. **Offer fix plan**:
    If any issues or suggestions were found, ask the user if they want a fix plan. If yes, delegate to compose: `skill: "compose", args: "plan fixes from the review findings"`.

## Review Checklist

### Test Coverage
- Are exported/public functions covered by tests?
- Are important edge cases tested (empty inputs, boundary values, error paths)?
- Do tests assert behavior and outcomes, not implementation details?
- Do tests exercise the actual code path, or do they bypass it by manually constructing expected state?
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
