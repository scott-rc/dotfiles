# Skill Templates

Annotated templates for creating skill files. Replace placeholders (`<...>`) with actual content.

**Default to the simple template.** Use the hub-and-spoke template only when a concrete extraction trigger is met: an operation has conditional branches, reads shared reference files, delegates to a subagent, or multiple operations together make SKILL.md hard to scan as a router.

## Simple Skill Template

For skills with one or a few linear operations that need no shared references or agent delegation. Everything lives in SKILL.md.

```markdown
---
name: <skill-name>
description: <Third-person sentence. What it does AND when to use it. Include trigger keywords.>
# Optional fields (include only what applies):
# disable-model-invocation: true
# user-invocable: false
# allowed-tools: Read, Grep, Glob
# model: sonnet
# context: fork
# agent: Explore
# argument-hint: "[issue-number]"
---

# <Skill Title>

<One sentence: what this skill helps Claude do.>

## Operations

### <Operation Name>

<One-line summary.>

1. **<Step name>**: <What to do.>
2. **<Step name>**: <What to do.>
3. **<Step name>**: MUST report results to the user. <Specify what to include.>

### <Operation Name>

<One-line summary.>

1. **<Step name>**: <What to do.>
2. **<Step name>**: <What to do.>
```

## Hub-and-Spoke SKILL.md Template

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
MUST read operations/<operation-file>.md before executing.

### <Operation Name>
<One-line summary.>
MUST read operations/<operation-file>.md before executing.

## Combined Operations

Users often request multiple operations together. Handle these as follows:

- **"<user phrase>"** → <Which operations to run, in what order>
- **"<user phrase>"** → <Which operations to run>

## References

These files are referenced by the operation instructions above:

- references/<reference-file>.md - <One-line description>
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
   MUST read references/<reference-file>.md for <what detail> before proceeding.

4. **<Step name>**:
   MUST report results to the user. <Specify what to include.>
```
