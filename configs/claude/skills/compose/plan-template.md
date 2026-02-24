# Plan Templates

Templates for plan artifacts. Replace `<...>` placeholders with actual content.

## Master Plan Template

```markdown
# <Plan Title>

<2-3 sentence summary of what this plan accomplishes and why.>

## Chunks

1. [chunk-01-<slug>.md](chunk-01-<slug>.md) -- <one-line description>
2. [chunk-02-<slug>.md](chunk-02-<slug>.md) -- <one-line description>
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

## Chunk Writer Subagent Inputs

The chunk-writer agent's system prompt is in `chunk-writer.md`. The orchestrator supplies these four sections per chunk:

- **Chunk Details** — number, title, dependency, and 2-4 sentence summary
- **High-Level Steps** — numbered list from the decomposition
- **Codebase Context** — file paths, function/type names, patterns, conventions
- **Build and Test** — build and test commands

## Chunking Guidelines

Follow these when decomposing a task into chunks:

- **Refactor first** -- if the task requires new abstractions or restructuring, make chunk 01 a pure refactor with no behavior change. This gives later chunks a clean foundation.
- **One feature per chunk** -- each chunk should add exactly one user-visible capability or complete one logical unit of work. Do not mix unrelated changes.
- **Buildable after each** -- the codebase MUST build and pass tests after every chunk completes. Never leave the codebase in a broken intermediate state.
- **~15-25 checkboxes per chunk** -- enough for meaningful progress, few enough to complete in one Claude Code session. If a chunk exceeds 25, split it.
- **Declare dependencies** -- each chunk's "Depends on" line names the chunk file it requires. Chunk 01 depends on "None". Keep the dependency chain linear when possible.
- **Test first when testable** -- for chunks adding testable behavior, structure step groups as red-green-refactor: "Red" (write failing tests), "Green" (implement to pass), "Refactor" (clean up). Include explicit test-run checkboxes to confirm failure then success. Chunks that are pure refactoring, config, or glue code use plain step groups.
- **Docs and cleanup last** -- put documentation updates, README changes, and cleanup in the final chunk. Earlier chunks focus on implementation.
- **Independently verifiable** -- each chunk's Verification section should confirm its work without relying on later chunks. A reviewer should be able to check one chunk in isolation.
