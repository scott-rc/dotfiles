# Quality Checklist

Pass/fail criteria for evaluating Claude Code skills and rules files. Each item is either PASS or FAIL. A skill or rules file must pass all items in a category to pass that category.

## Core Quality

- [ ] **Description specificity** *(Skills only)*: The `description` field names concrete actions and triggers, not vague capabilities (FAIL: "Helps with various tasks")
- [ ] **Line count** *(Skills only)*: SKILL.md is under 500 lines and under 5000 tokens
- [ ] **Terminology consistency**: The same concept uses the same word everywhere (e.g., always "operation" or always "command", never both)
- [ ] **Progressive disclosure** *(Skills only)*: SKILL.md summarizes, operation files detail, reference files go deep -- no level repeats information from another
- [ ] **Examples where needed** *(Skills only)*: Operations that produce formatted output (commit messages, PR descriptions, file scaffolds) include at least one example
- [ ] **Verification method** *(Skills only)*: Task-oriented operations include a step for verifying results (run tests, check output, compare screenshots)
- [ ] **Invocation control** *(Skills only)*: Skills with side effects use `disable-model-invocation: true`; background-knowledge skills use `user-invocable: false`; default is appropriate for dual-invocation skills

## Structure — Skills

- [ ] **Valid frontmatter**: SKILL.md has YAML frontmatter with required `name` and `description` fields and only valid optional fields
- [ ] **Name matches directory**: The `name` in frontmatter matches the directory name exactly
- [ ] **Naming rules**: Skill name and all file names are lowercase with hyphens only, max 64 characters
- [ ] **Operations section exists**: SKILL.md has an H2 "Operations" section with at least one operation
- [ ] **Operation files exist**: Every operation listed in SKILL.md has a corresponding `.md` file
- [ ] **Reference files exist**: Every file linked from SKILL.md or operation files exists
- [ ] **No orphan files**: Every `.md` file in the directory is referenced from SKILL.md or an operation file
- [ ] **H1 headings match**: Each operation file's H1 starts with the operation name from SKILL.md
- [ ] **Companion agent files exist**: Every agent name referenced in an operation's Task tool call (via `agent:` field) has a corresponding file in `configs/claude/agents/<name>.md`

## Structure — Rules

- [ ] **Appropriate file location**: The rules file is in the correct location for its scope (project root, subdirectory, global, or `.claude/rules/`)
- [ ] **@file references resolve**: Every `@filename` reference points to a file that exists
- [ ] **No content duplication**: Information in referenced files (`@README.md`, etc.) is not repeated in the rules file
- [ ] **Scoped rules have paths**: Files in `.claude/rules/` intended to be path-specific have `paths:` frontmatter with valid glob patterns. Files without `paths:` load unconditionally (this is valid for topic-specific rules that apply project-wide).
- [ ] **Flat heading hierarchy**: Headings do not go deeper than H3

## Content Efficiency

- [ ] **Token justification**: Every file contributes unique information -- no file exists just for organizational aesthetics
- [ ] **No redundancy**: Instructions are stated once and referenced, not copied between files
- [ ] **No over-explaining**: Steps don't explain basic concepts Claude already knows (e.g., "markdown is a formatting language")
- [ ] **Concise steps** *(Skills only)*: Operation steps are actionable instructions, not essays. Each step should be 1-3 sentences.
- [ ] **Tight prose**: Terse, imperative style
- [ ] **No tables**: Lists instead of markdown tables
- [ ] **Only novel information** *(Rules only)*: Every instruction teaches something Claude cannot infer from the codebase or common knowledge
- [ ] **Actionable instructions** *(Rules only)*: Every instruction is specific enough to act on (FAIL: "write clean code", "follow best practices")
- [ ] **Conciseness test** *(Rules only)*: For each line, "would removing this cause Claude to make mistakes?" If no, it should be cut
- [ ] **Not over-specified** *(Rules only)*: File is not so long that important rules get lost — if Claude ignores rules despite them being present, the file needs pruning

## Scripts (if applicable, Skills only)

- [ ] **Error handling**: Scripts check for failure conditions and provide useful error messages rather than failing silently
- [ ] **Error recovery**: Scripts handle errors with concrete recovery actions rather than surfacing raw errors for Claude to interpret
- [ ] **Documented constants**: Magic numbers and paths are explained or assigned to named variables
- [ ] **Dependencies declared**: Required tools are documented in the skill
- [ ] **POSIX paths**: Scripts use forward slashes only

## Workflow Quality (Skills only)

- [ ] **Sequential steps**: Operations use numbered steps that flow logically from start to finish
- [ ] **Decision points**: Conditional branches are explicit ("If X, do Y. Otherwise, do Z.")
- [ ] **Error cases**: Operations handle likely failure modes (missing files, invalid input, conflicts)
- [ ] **Feedback to user**: Operations tell Claude when to report progress or results to the user
- [ ] **Feedback loops**: Quality-critical operations include a validate-fix-repeat loop (e.g., run linter, fix errors, re-run)
- [ ] **Degrees of freedom**: Each step's specificity matches its fragility -- fragile/critical steps are prescriptive, variable/creative steps leave room
- [ ] **RFC keyword usage**: MUST/SHOULD/MAY keywords are used to distinguish hard requirements from recommendations and optional behavior
- [ ] **Subagent delegation**: Operations delegate work (file reading, analysis, artifact generation) to subagents rather than performing it inline in the orchestrator's context. The orchestrator handles user interaction and decision-making only.
- [ ] **Named agents for reuse**: When the same delegation prompt appears in two or more operations, it is extracted into a named agent file rather than duplicated inline

## Rules Quality

- [ ] **Appropriate granularity**: CLAUDE.md files under ~200 lines; split into scoped rules or `@file` references if longer
- [ ] **No common knowledge**: Does not teach Claude things it already knows (language syntax, standard library, well-known patterns)
- [ ] **No README duplication**: Uses `@README.md` instead of copying project setup information
- [ ] **Correct scope placement**: Instructions that apply to a subset of the codebase use scoped rules, not the main CLAUDE.md

## Testing (Skills only)

- [ ] **Tested with target models**: The skill has been tested with the models it targets
- [ ] **Evaluation cases exist**: At least one test scenario per operation exists to verify correct behavior
- [ ] **Structured evaluations**: Test scenarios specify input, expected behavior, and pass/fail criteria -- not just vague descriptions

## Anti-patterns (FAIL if any are present)

### Shared Anti-patterns

- [ ] **No time-sensitive content**: No version numbers, dates, or URLs that will rot
- [ ] **No inconsistent terms**: Same concept uses the same word everywhere
- [ ] **No Windows paths**: POSIX paths only, forward slashes

### Skill Anti-patterns

- [ ] **No nested references**: Reference files MUST NOT reference other reference files
- [ ] **No vague file names**: No files named `utils.md`, `helpers.md`, `misc.md`, or `other.md`
- [ ] **No SKILL.md instructions**: SKILL.md routes to operations, does not contain inline instructions
- [ ] **No unbounded output**: Operations that produce output specify length limits or truncation rules
- [ ] **No unprompted options**: Operations do not present multiple approaches when one clear default will do
- [ ] **No keyword inflation**: MUST is not applied to every rule indiscriminately — if most rules use MUST, the skill needs reclassification
- [ ] **No reference-only fork skills**: Skills with `context: fork` contain task instructions, not just reference content (reference content needs no fork)
- [ ] **No cross-skill file references**: Use the Skill tool for cross-skill delegation, not relative file paths
- [ ] **No inline system prompts for reusable agents**: Operations do not embed full system prompts in ad-hoc Task tool delegation when the same agent identity (prompt + tools + model) is reused across two or more invocations -- extract into a named agent file instead
- [ ] **No inline heavy work**: Operations do not read many files, analyze diffs, or generate artifacts inline when a subagent could do it -- orchestrator context is for user interaction and decision-making

