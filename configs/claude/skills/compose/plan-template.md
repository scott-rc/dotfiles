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

## Orchestrator Prompt

\`\`\`
<Paste the orchestrator prompt here -- see Orchestrator Prompt Template below.>
\`\`\`
```

## Chunk File Template

```markdown
# Chunk <NN>: <Title>

**Depends on**: <chunk-NN-slug.md, or "None">

## What and Why

<2-4 sentences explaining what this chunk does and why it matters. Include enough context that a fresh agent session can understand the purpose without reading other chunks.>

## Implementation Steps

### 1. <Step Group Name>

- [ ] <Concrete action with file paths, function names, or shell commands>
- [ ] <Next action>
- [ ] ...

### 2. <Step Group Name>

- [ ] <Action>
- [ ] ...

## Verification

- [ ] <Build command passes>
- [ ] <Test command passes>
- [ ] <Manual check or assertion>
```

## Orchestrator Prompt Template

This prompt drives sequential execution of all chunks. It goes inside the master plan's "Orchestrator Prompt" code block.

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
2. For each chunk in order, launch a Task tool subagent (type: general-purpose) with the Chunk Subagent Prompt below, substituting the chunk file path
3. After each subagent completes, verify it reported success. If it reported failure, STOP and report the failure to the user -- do not continue to the next chunk.
4. After all chunks complete, report the full plan as done

# Anti-requirements

- Do NOT run chunks in parallel -- each chunk may depend on the previous one
- Do NOT skip chunks, even if they look already done -- the subagent will detect completion and return immediately
- Do NOT modify chunk files yourself -- only subagents modify them

# Chunk Subagent Prompt

Use this as the prompt for each Task subagent, replacing CHUNK_FILE_PATH:

---

Execute the implementation chunk defined in CHUNK_FILE_PATH.

## Process

1. Read the chunk file
2. Check the "Depends on" line. If it names a dependency, read that chunk file and verify all its checkboxes are checked. If any are unchecked, STOP and report the dependency is incomplete.
3. Scan for the first unchecked `- [ ]` item. If all items are checked, report "Chunk already complete" and stop.
4. Starting from that item, execute each unchecked step in order:
   a. Perform the action described
   b. Mark the checkbox: replace `- [ ]` with `- [x]` in the chunk file
   c. After completing each numbered step group, run the build command and fix any errors before moving on
5. After all Implementation Steps are checked, execute the Verification section the same way
6. When all checkboxes are checked, report "Chunk complete"

## Rules

- If a step fails and you cannot fix it within 3 attempts, STOP and report the failure with details
- MUST mark each checkbox immediately after completing it -- this is the progress ledger
- MUST NOT skip ahead or reorder steps
- MUST run the build command after each step group, not just at the end

---
```

## Chunking Guidelines

Follow these when decomposing a task into chunks:

- **Refactor first** -- if the task requires new abstractions or restructuring, make chunk 01 a pure refactor with no behavior change. This gives later chunks a clean foundation.
- **One feature per chunk** -- each chunk should add exactly one user-visible capability or complete one logical unit of work. Do not mix unrelated changes.
- **Buildable after each** -- the codebase MUST build and pass tests after every chunk completes. Never leave the codebase in a broken intermediate state.
- **~15-25 checkboxes per chunk** -- enough for meaningful progress, few enough to complete in one agent session. If a chunk exceeds 25, split it.
- **Declare dependencies** -- each chunk's "Depends on" line names the chunk file it requires. Chunk 01 depends on "None". Keep the dependency chain linear when possible.
- **Docs and cleanup last** -- put documentation updates, README changes, and cleanup in the final chunk. Earlier chunks focus on implementation.
- **Independently verifiable** -- each chunk's Verification section should confirm its work without relying on later chunks. A reviewer should be able to check one chunk in isolation.
