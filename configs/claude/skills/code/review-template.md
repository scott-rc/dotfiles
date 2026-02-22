# Review Templates

Templates for review subagent prompts. Replace `<...>` placeholders with actual content.

## Review Subagent Prompt Template

Use this prompt when spawning each review subagent in step 10 of review.md.

```markdown
Review the following scope of code changes.

## Scope

- **Name**: <scope-name>
- **Files**:
<file-list, one per line>
- **Focus**: <what to pay attention to in this scope>
- **Scope-specific criteria**: <additional criteria beyond the standard checklist>

## Guidelines

Read and apply these guidelines:
- <path-to-general-guidelines.md>
- <path-to-testing-guidelines.md>
- <path-to-language-specific-guidelines.md, if applicable — omit if not applicable>

## Project Context

<repo root, key paths, conventions, patterns, and project-specific standards observed during guideline/context loading>

## Review Checklist

Evaluate every item below. Track findings with severity:
- **issue** — likely bug, missing error handling at a boundary, or correctness problem
- **suggestion** — improvement that makes code clearer, simpler, or more maintainable
- **nit** — minor style or preference item

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

## Output Format

Report findings grouped by severity (issues first, then suggestions, then nits). Each finding MUST include:
- `file_path:line_number`
- What the problem is (one sentence)
- A concrete fix or recommendation

If no findings in this scope, say so — do not manufacture issues.
```

## Consolidated Report Format

Structure for the merged findings report after all subagents complete.

```markdown
# Review Findings

## Issues

- **`<file_path>:<line>`** — <problem description>
  Fix: <concrete fix or recommendation>

## Suggestions

- **`<file_path>:<line>`** — <problem description>
  Fix: <concrete fix or recommendation>

## Nits

- **`<file_path>:<line>`** — <problem description>
  Fix: <concrete fix or recommendation>
```
