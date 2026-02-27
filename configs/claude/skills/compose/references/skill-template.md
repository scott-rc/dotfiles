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

<One sentence: what this skill helps Claude do.>

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

## References

These files are referenced by the operation instructions above:

- [<reference-file>.md](references/<reference-file>.md) - <One-line description>
```

## Operation File Template

```markdown
# <Name> (or # <Descriptive Phrase> starting with the operation name)

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
   MUST read [<reference-file>.md](references/<reference-file>.md) for <what detail> before proceeding.

4. **<Step name>**:
   MUST report results to the user. <Specify what to include.>
```

## Naming Guidance

When choosing a skill name, prefer:
- **Gerund form** when natural: `managing-deploys`, `reviewing-code`, `syncing-data`
- **Domain noun** when gerund is awkward: `git`, `docker`, `kubernetes`
- **Hyphenated compound** for specificity: `pr-review`, `test-runner`, `db-migrations`
