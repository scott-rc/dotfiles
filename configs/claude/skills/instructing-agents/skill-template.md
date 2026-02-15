# Skill Templates

Annotated templates for creating skill files. Replace placeholders (`<...>`) with actual content.

## SKILL.md Template

```markdown
---
name: <skill-name>
description: <Third-person sentence. What it does AND when to use it. Include trigger keywords.>
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
- **Gerund form** when natural: `instructing-agents`, `managing-deploys`, `reviewing-code`
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
