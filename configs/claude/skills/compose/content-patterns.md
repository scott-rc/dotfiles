# Content Patterns

Reusable patterns for structuring operation steps, task skills, and dynamic context injection.

## Content Patterns for Operation Steps

Use these patterns inside operation files when they fit the task:

- **Checklist pattern**: Give Claude a checklist to copy and track progress through multi-step work. Useful when steps can partially succeed.
- **Feedback loop pattern**: Run validator/linter/tests, fix errors, repeat until clean. Dramatically improves output quality for code-generation or formatting tasks.
- **Template pattern**: Provide a strict output template (low freedom) or a flexible one with optional sections (medium freedom).
- **Examples pattern**: Show 1-2 input/output pairs when the desired style or format is ambiguous.
- **Conditional routing pattern**: "If X, go to step N. If Y, go to step M." Use when an operation has meaningfully different paths.
- **Authority pattern**: Use RFC 2119 keywords (MUST, SHOULD, MAY) to signal instruction priority. Claude complies more reliably with capitalized directive keywords. Reserve MUST for steps where skipping breaks the workflow.
- **Interview pattern**: When an operation needs user input before proceeding, batch related questions into a single step rather than asking one at a time. Ask only what's needed to unblock the next decision -- don't front-load every possible question. Use follow-up rounds to drill into areas the user's answers reveal as complex or ambiguous. End the interview by summarizing your understanding and confirming before proceeding. Use this pattern when requirements are unclear, the domain has many valid options, or the user's initial request is vague.
- **Cross-skill delegation pattern**: When an operation needs functionality from another skill, use the Skill tool to load it (`skill: "<name>"`) rather than referencing files across skill directories with relative paths. This formally loads the other skill's SKILL.md and makes its references available. For full workflow delegation, include routing args (e.g., `skill: "compose", args: "plan this task"`). For loading references only, invoke without operation-specific args and direct Claude to read specific items from the loaded skill's References section.

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

## Delivery Pattern

When the final output is a prompt or text artifact for the user to paste into another session:

1. Scan for non-ASCII characters and replace with ASCII equivalents ("--" for em dashes, "->" for arrows, straight quotes for smart quotes, "..." for ellipses). Non-ASCII corrupts into mojibake when pasted across sessions.
2. Print the result inside a markdown code block.
3. Copy to the clipboard via `pbcopy`.
4. Tell the user the content is copied and ready to paste.

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
