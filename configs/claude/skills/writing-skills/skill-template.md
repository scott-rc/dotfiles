# Skill Templates

Annotated templates for creating skill files. Replace placeholders (`<...>`) with actual content.

## SKILL.md Template

```markdown
---
name: <skill-name>
description: <Third-person sentence. What it does AND when to use it. Include trigger keywords.>
compatibility: <Optional. Runtime requirements like CLIs or APIs.>
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

**Important**: For each operation, read and follow its detailed instruction file.

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
   <What to do. Be specific and actionable.>

2. **<Step name>**:
   <What to do.>
   - If <condition>: <action>
   - Otherwise: <action>

3. **<Step name>**:
   <What to do. Reference shared knowledge:>
   See [<reference-file>.md](<reference-file>.md) for <what detail>.

4. **<Step name>**:
   <Report results to the user. Specify what to include.>
```

## Naming Guidance

When choosing a skill name, prefer:
- **Gerund form** when natural: `writing-skills`, `managing-deploys`, `reviewing-code`
- **Domain noun** when gerund is awkward: `git`, `docker`, `kubernetes`
- **Hyphenated compound** for specificity: `pr-review`, `test-runner`, `db-migrations`
