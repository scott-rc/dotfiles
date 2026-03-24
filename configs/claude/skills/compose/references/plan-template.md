# Plan Templates

Templates for plan artifacts. Replace `<...>` placeholders with actual content.

## Master Plan Template

```markdown
# <Plan Title>

<2-3 sentence summary of what this plan accomplishes and why.>

## Chunks

1. chunk-01-<slug>.md -- <one-line description>
2. chunk-02-<slug>.md -- <one-line description>
...
```

## Chunk File Template

The canonical chunk file format is defined in references/chunk-format.md. Chunks use one of two structures:

- **TDD chunk** (adding testable behavior): Red-green-refactor step groups with explicit test-run checkboxes
- **Non-TDD chunk** (refactoring, config, glue): Plain numbered step groups

Both include: a "Depends on" line, a "What and Why" section, numbered "Implementation Steps" with `- [ ]` checkboxes, and a "Verification" section. Target ~15-25 checkboxes per chunk.

## Orchestrator Prompt Template

This prompt drives sequential execution of all chunks. It is written to the Claude Code plan file via plan mode so the user can accept and begin execution immediately.

```markdown
# Goal

Execute the plan in ./tmp/<plan-name>/plan.md by running each chunk as a sequential subagent.

# Context

- Plan file: ./tmp/<plan-name>/plan.md
- Chunk files: ./tmp/<plan-name>/chunk-*.md
- Codebase: <root directory and key paths>
- Build: <build command>
- Test: <test command>

# Requirements

1. Read plan.md to get the ordered chunk list
2. For each chunk in order:
   a. Read the chunk file
   b. Check the "Depends on" line -- if it names a dependency, verify all its checkboxes are checked. If any are unchecked, STOP and report.
   c. Execute each unchecked `- [ ]` step in order: perform the action, then mark the checkbox `- [x]`
   d. After completing each numbered step group, run the build command and fix errors (max 3 attempts)
   e. After all Implementation Steps, execute each Verification checkbox
3. After each chunk completes, verify success. If a step fails after 3 attempts, STOP and report -- do not continue to the next chunk.
4. After all chunks complete, report the full plan as done

# Rules

- Do NOT run chunks in parallel -- each chunk may depend on the previous one
- Do NOT skip chunks, even if they look already done -- check for unchecked boxes
- MUST mark each checkbox immediately after completing it
- MUST NOT skip ahead or reorder steps
- For TDD chunks (Red/Green/Refactor), verify test failures before implementing and test passes after
- Execute steps as written -- do not re-evaluate design decisions made during planning
```
