# Quality Checklist

Pass/fail criteria for evaluating Claude Code skills and rules files. Items are tiered by severity.

- **Blocking** — MUST pass. Breaks functionality if violated.
- **Important** — SHOULD fix. Degrades quality.
- **Cosmetic** — MAY fix. Consistency polish.

Only blocking items can fail a review. Important items are flagged. Cosmetic items are noted.

## Structure — Skills

### Blocking

- [ ] **Valid frontmatter**: SKILL.md has YAML frontmatter with required `name` and `description` fields and only valid optional fields
- [ ] **Name matches directory**: The `name` in frontmatter matches the directory name exactly
- [ ] **Naming rules**: Names follow [skill-spec.md](skill-spec.md) naming rules
- [ ] **Operations section exists**: SKILL.md has an H2 "Operations" section with at least one operation
- [ ] **Operation files exist**: Every operation listed in SKILL.md that links to a file has a corresponding `.md` file
- [ ] **Reference files exist**: Every file linked from SKILL.md or operation files exists
- [ ] **No orphan files**: Every `.md` file in the directory is referenced from SKILL.md or an operation file
- [ ] **H1 headings match**: Each operation file's H1 starts with the operation name from SKILL.md
- [ ] **No cross-skill file references**: MUST NOT reference another skill's files via relative paths. Use the Skill tool for cross-skill delegation.
- [ ] **No nested references**: Reference files MUST NOT reference other reference files
- [ ] **No circular references**: No file directly or indirectly references itself
- [ ] **Line count**: SKILL.md is under 500 lines and under 5000 tokens
- [ ] **Companion agent files exist**: Every agent name referenced in an operation's Task tool call (via `agent:` field) has a corresponding file in `configs/claude/agents/<name>.md`
- [ ] **No unbounded output**: Operations that produce output specify length limits or truncation rules
- [ ] **No SKILL.md inline complexity**: SKILL.md MAY contain simple inline operations (linear, no file refs, no branching). Complex operations with conditional logic, file references, or agent delegation MUST be in their own files.
- [ ] **No reference-only fork skills**: Skills with `context: fork` contain task instructions, not just reference content
- [ ] **Verification step (state-mutating)**: State-mutating operations (commit, deploy, write files) include a step for verifying results (run tests, check output, compare before/after)
- [ ] **Error handling (state-mutating)**: State-mutating operations (commit, deploy, write files) handle likely failure modes (missing files, invalid input, conflicts) rather than failing silently
- [ ] **No inline heavy work**: Operations do not read many files, analyze diffs, or generate artifacts inline when a subagent could do it

### Important

- [ ] **Verification step (non-mutating)**: Read-only or informational operations include a step for verifying results where practical
- [ ] **Feedback loops**: Quality-critical operations include a validate-fix-repeat loop (e.g., run linter, fix errors, re-run)
- [ ] **Examples where needed**: Operations that produce formatted output (commit messages, PR descriptions, file scaffolds) include at least one example
- [ ] **DRY references**: Reference files meet the DRY threshold in [skill-spec.md](skill-spec.md)
- [ ] **Terminology consistency**: The same concept uses the same word everywhere (e.g., always "operation" or always "command", never both)
- [ ] **Description specificity**: The `description` field names concrete actions and triggers, not vague capabilities (FAIL: "Helps with various tasks")
- [ ] **RFC keyword usage**: Operations use MUST/SHOULD/MAY to distinguish requirement levels. Neither all-MUST (overuse) nor all-plain-prose (under-use).
- [ ] **Sequential steps**: Operations use numbered steps that flow logically from start to finish
- [ ] **Decision points**: Conditional branches are explicit ("If X, do Y. Otherwise, do Z.")
- [ ] **Self-contained operations**: Operation files are understandable on their own; referenced files provide detail, not essential context (per [skill-spec.md](skill-spec.md))
- [ ] **Invocation control**: Skills with side effects use `disable-model-invocation: true`; background-knowledge skills use `user-invocable: false`; default is appropriate for dual-invocation skills
- [ ] **No vague file names**: No files named `utils.md`, `helpers.md`, `misc.md`, or `other.md`
- [ ] **No unprompted options**: Operations do not present multiple approaches when one clear default will do

### Cosmetic

- [ ] **H1 naming conventions**: Operation file H1s follow the pattern `# <Name> Operation`
- [ ] **Section heading consistency**: All operation files use the same heading structure
- [ ] **Tight prose**: Terse, imperative style
- [ ] **No tables**: Lists instead of markdown tables
- [ ] **Degrees of freedom matching**: Each step's specificity matches its fragility — fragile/critical steps are prescriptive, variable/creative steps leave room
- [ ] **Feedback to user**: Operations with 5+ steps or subagent calls include at least one intermediate progress report to the user

## Structure — Rules

### Blocking

- [ ] **Appropriate file location**: The rules file is in the correct location for its scope (project root, subdirectory, global, or `.claude/rules/`)
- [ ] **@file references resolve**: Every `@filename` reference points to a file that exists
- [ ] **Flat heading hierarchy**: Headings do not go deeper than H3

### Important

- [ ] **No content duplication**: Information in referenced files (`@README.md`, etc.) is not repeated in the rules file
- [ ] **Scoped rules have paths**: Files in `.claude/rules/` intended to be path-specific have `paths:` frontmatter with valid glob patterns
- [ ] **Appropriate granularity**: CLAUDE.md files under ~200 lines; split into scoped rules or `@file` references if longer
- [ ] **Correct scope placement**: Instructions that apply to a subset of the codebase use scoped rules, not the main CLAUDE.md
- [ ] **No common knowledge**: Does not teach Claude things it already knows (language syntax, standard library, well-known patterns)
- [ ] **No README duplication**: Uses `@README.md` instead of copying project setup information
- [ ] **Actionable instructions**: Every instruction is specific enough to act on (FAIL: "write clean code", "follow best practices")
- [ ] **Conciseness test**: For each line, "would removing this cause Claude to make mistakes?" If no, cut it

## Content Efficiency

### Important

- [ ] **Token justification**: Every file contributes unique information — no file exists just for organizational aesthetics
- [ ] **No redundancy**: Instructions are stated once and referenced, not copied between files
- [ ] **Concise steps** *(Skills only)*: Operation steps are actionable instructions, not essays. Each step should be 1-3 sentences.
- [ ] **No over-explaining**: Steps don't explain basic concepts Claude already knows (e.g., "markdown is a formatting language")
- [ ] **No time-sensitive content**: No version numbers, dates, or URLs that will rot
- [ ] **POSIX paths**: Forward slashes only, no Windows paths

## Scripts (if applicable, Skills only)

### Important

- [ ] **Error handling**: Scripts check for failure conditions and provide useful error messages rather than failing silently
- [ ] **Error recovery**: Scripts handle errors with concrete recovery actions rather than surfacing raw errors for Claude to interpret
- [ ] **Dependencies declared**: Required tools are documented in the skill

### Cosmetic

- [ ] **Documented constants**: Magic numbers and paths are explained or assigned to named variables

## Testing (Skills only)

### Important

- [ ] **Tested with target models**: The skill has been tested with the models it targets
- [ ] **Evaluation cases exist**: At least one test scenario per operation exists to verify correct behavior
- [ ] **Structured evaluations**: Test scenarios specify input, expected behavior, and pass/fail criteria — not just vague descriptions
