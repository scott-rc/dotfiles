# Quality Checklist

Pass/fail criteria for evaluating a Claude Code skill. Each item is either PASS or FAIL. A skill must pass all items in a category to pass that category.

## Core Quality

- [ ] **Description specificity**: The `description` field names concrete actions and triggers, not vague capabilities (FAIL: "Helps with various tasks")
- [ ] **Line count**: SKILL.md is under 500 lines and under 5000 tokens
- [ ] **Terminology consistency**: The same concept uses the same word everywhere (e.g., always "operation" or always "command", never both)
- [ ] **Progressive disclosure**: SKILL.md summarizes, operation files detail, reference files go deep -- no level repeats information from another
- [ ] **Examples where needed**: Operations that produce formatted output (commit messages, PR descriptions, file scaffolds) include at least one example

## Structure

- [ ] **Valid frontmatter**: SKILL.md has YAML frontmatter with required `name` and `description` fields
- [ ] **Name matches directory**: The `name` in frontmatter matches the directory name exactly
- [ ] **Naming rules**: Skill name and all file names are lowercase with hyphens only, max 64 characters
- [ ] **Operations section exists**: SKILL.md has an H2 "Operations" section with at least one operation
- [ ] **Operation files exist**: Every operation listed in SKILL.md has a corresponding `.md` file
- [ ] **Reference files exist**: Every file linked from SKILL.md or operation files exists
- [ ] **No orphan files**: Every `.md` file in the directory is referenced from SKILL.md or an operation file
- [ ] **H1 headings match**: Each operation file's H1 starts with the operation name from SKILL.md

## Content Efficiency

- [ ] **Token justification**: Every file contributes unique information -- no file exists just for organizational aesthetics
- [ ] **No redundancy**: Instructions are stated once and referenced, not copied between files
- [ ] **No over-explaining**: Steps don't explain basic concepts the agent already knows (e.g., "markdown is a formatting language")
- [ ] **Concise steps**: Operation steps are actionable instructions, not essays. Each step should be 1-3 sentences.

## Scripts (if applicable)

- [ ] **Error recovery**: Scripts handle errors with concrete recovery or useful messages rather than failing silently
- [ ] **Solve, don't punt**: Scripts handle errors with concrete recovery actions rather than surfacing raw errors for the agent to interpret
- [ ] **Error handling**: Scripts check for failure conditions and report useful error messages
- [ ] **Documented constants**: Magic numbers and paths are explained or assigned to named variables
- [ ] **Dependencies declared**: Required tools are documented in the skill
- [ ] **POSIX paths**: Scripts use forward slashes only

## Workflow Quality

- [ ] **Sequential steps**: Operations use numbered steps that flow logically from start to finish
- [ ] **Decision points**: Conditional branches are explicit ("If X, do Y. Otherwise, do Z.")
- [ ] **Error cases**: Operations handle likely failure modes (missing files, invalid input, conflicts)
- [ ] **Feedback to user**: Operations tell the agent when to report progress or results to the user
- [ ] **Feedback loops**: Quality-critical operations include a validate-fix-repeat loop (e.g., run linter, fix errors, re-run)
- [ ] **Degrees of freedom**: Each step's specificity matches its fragility -- fragile/critical steps are prescriptive, variable/creative steps leave room

## Testing

- [ ] **Tested with target models**: The skill has been tested with the models it targets
- [ ] **Evaluation cases exist**: At least one test scenario per operation exists to verify correct behavior
- [ ] **Structured evaluations**: Test scenarios specify input, expected behavior, and pass/fail criteria -- not just vague descriptions

## Anti-patterns (FAIL if any are present)

- [ ] **No nested references**: Reference files do not link to other reference files
- [ ] **No vague file names**: No files named `utils.md`, `helpers.md`, `misc.md`, or `other.md`
- [ ] **No Windows paths**: No backslashes in file paths
- [ ] **No time-sensitive content**: No specific version numbers, dates, or URLs that will rot
- [ ] **No inconsistent terms**: The same concept is not called by different names in different files
- [ ] **No SKILL.md instructions**: SKILL.md routes to operation files, it does not contain step-by-step instructions itself
- [ ] **No unbounded output**: Operations that produce output specify length limits or truncation rules
- [ ] **No unprompted options**: Operations do not present multiple approaches when one clear default will do
