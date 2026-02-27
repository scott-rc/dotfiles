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

The canonical chunk file format is defined in the chunk-writer agent's Output Format section. Chunks use one of two structures:

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
2. For each chunk in order, launch a Task tool subagent (type: chunk-executor) with the prompt below, substituting CHUNK_FILE_PATH
3. After each subagent completes, verify it reported success. If it reported failure, STOP and report the failure to the user -- do not continue to the next chunk.
4. After all chunks complete, report the full plan as done

# Anti-requirements

- Do NOT run chunks in parallel -- each chunk may depend on the previous one
- Do NOT skip chunks, even if they look already done -- the subagent will detect completion and return immediately
- Do NOT modify chunk files yourself -- only subagents modify them

# Chunk Subagent Prompt

Use this as the prompt for each Task subagent, replacing CHUNK_FILE_PATH:

---

Execute the implementation chunk at CHUNK_FILE_PATH.

- Build command: <build command>
- Test command: <test command>

---
```
