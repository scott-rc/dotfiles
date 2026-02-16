# Skill Templates

Annotated templates for creating skill files. Replace placeholders (`<...>`) with actual content.

## SKILL.md Template

```markdown
---
name: <skill-name>
description: <Third-person sentence. What it does AND when to use it. Include trigger keywords.>
# Optional fields (include only what applies):
# disable-model-invocation: true    # Only user can invoke (for side-effect workflows)
# user-invocable: false             # Only Claude can invoke (for background knowledge)
# allowed-tools: Read, Grep, Glob   # Restrict available tools
# model: sonnet                     # Model override
# context: fork                     # Run in subagent isolation
# agent: Explore                    # Subagent type (requires context: fork)
# argument-hint: "[issue-number]"   # Autocomplete hint
---

# <Skill Title>

<One sentence: what this skill helps the agent do.>

## Operations

### <Operation Name>
<One-line summary of what this operation does.>
See [<operation-file>.md](<operation-file>.md) for detailed instructions.

### <Operation Name>
<One-line summary.>
See [<operation-file>.md](<operation-file>.md) for detailed instructions.

## Combined Operations

Users often request multiple operations together. Handle these as follows:

- **"<user phrase>"** → <Which operations to run, in what order>
- **"<user phrase>"** → <Which operations to run>

**Important**: You MUST read and follow the detailed operation file for each operation before executing it. Do not rely on the summaries above.

## References

These files are referenced by the operation instructions above:

- [<reference-file>.md](<reference-file>.md) - <One-line description>
```

## Operation File Template

```markdown
# <Operation Name>

<One sentence describing what this operation does and its outcome.>

## Instructions

1. **<Step name>**:
   <What to do. Be specific and actionable. Use MUST for hard requirements, SHOULD for recommendations.>

2. **<Step name>**:
   <What to do.>
   - If <condition>: <action>
   - Otherwise: <action>

3. **<Step name>**:
   <What to do. Reference shared knowledge:>
   MUST read [<reference-file>.md](<reference-file>.md) for <what detail> before proceeding.

4. **<Step name>**:
   MUST report results to the user. <Specify what to include.>
```

## Naming Guidance

When choosing a skill name, prefer:
- **Gerund form** when natural: `managing-deploys`, `reviewing-code`, `syncing-data`
- **Domain noun** when gerund is awkward: `git`, `docker`, `kubernetes`
- **Hyphenated compound** for specificity: `pr-review`, `test-runner`, `db-migrations`

## Content Patterns for Operation Steps

Use these patterns inside operation files when they fit the task:

- **Checklist pattern**: Give the agent a checklist to copy and track progress through multi-step work. Useful when steps can partially succeed.
- **Feedback loop pattern**: Run validator/linter/tests, fix errors, repeat until clean. Dramatically improves output quality for code-generation or formatting tasks.
- **Template pattern**: Provide a strict output template (low freedom) or a flexible one with optional sections (medium freedom).
- **Examples pattern**: Show 1-2 input/output pairs when the desired style or format is ambiguous.
- **Conditional routing pattern**: "If X, go to step N. If Y, go to step M." Use when an operation has meaningfully different paths.
- **Authority pattern**: Use RFC 2119 keywords (MUST, SHOULD, MAY) to signal instruction priority. Agents comply more reliably with capitalized directive keywords. Reserve MUST for steps where skipping breaks the workflow.
- **Interview pattern**: When an operation needs user input before proceeding, batch related questions into a single step rather than asking one at a time. Ask only what's needed to unblock the next decision — don't front-load every possible question. Use follow-up rounds to drill into areas the user's answers reveal as complex or ambiguous. End the interview by summarizing your understanding and confirming before proceeding. Use this pattern when requirements are unclear, the domain has many valid options, or the user's initial request is vague.

## Task Skill Pattern

Skills that run a specific workflow (deploys, migrations, data transforms) rather than augmenting knowledge. Combine `context: fork` for isolation with `disable-model-invocation: true` for safety.

```markdown
---
name: run-migration
description: Runs database migrations against the target environment when the user asks to migrate, apply migrations, or update the schema.
disable-model-invocation: true
context: fork
agent: general-purpose
---

# Run Migration

Run pending database migrations for $ARGUMENTS.

1. **Check current state**: Run `db migrate status` and report pending migrations
2. **Confirm with user**: List migrations that will run and ask for confirmation
3. **Apply**: Run `db migrate up` and capture output
4. **Verify**: Run `db migrate status` again to confirm all migrations applied
5. **Report**: Show applied migrations and any errors
```

## Dynamic Context Pattern

Use `` !`command` `` to inject runtime data into skill content. The command runs before content reaches Claude.

```markdown
---
name: review-changes
description: Reviews staged git changes when the user asks to review, check, or inspect their changes.
---

# Review Changes

Review the following staged changes:

!`git diff --cached`

Evaluate for correctness, style, and potential bugs.
```
