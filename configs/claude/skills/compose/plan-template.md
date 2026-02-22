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

For chunks adding testable behavior, structure step groups as red-green-refactor: write failing tests first, implement to make them pass, then refactor. For pure refactoring, config, or glue code, use plain step groups.

### TDD chunk (adding testable behavior)

```markdown
# Chunk <NN>: <Title>

**Depends on**: <chunk-NN-slug.md, or "None">

## What and Why

<2-4 sentences explaining what this chunk does and why it matters. Include enough context that a fresh Claude Code session can understand the purpose without reading other chunks.>

## Implementation Steps

### 1. Red: Write failing tests for <feature>

- [ ] <Create/update test file at path>
- [ ] <Write test for expected behavior>
- [ ] <Run tests -- confirm they fail for the right reason>

### 2. Green: Implement <feature>

- [ ] <Implement the minimal code to make tests pass>
- [ ] <Run tests -- confirm they pass>

### 3. Refactor

- [ ] <Clean up implementation if needed>
- [ ] <Run tests -- confirm they still pass>

## Verification

- [ ] <Build command passes>
- [ ] <Test command passes>
- [ ] <Manual check or assertion>
```

### Non-TDD chunk (refactoring, config, glue)

```markdown
# Chunk <NN>: <Title>

**Depends on**: <chunk-NN-slug.md, or "None">

## What and Why

<2-4 sentences explaining what this chunk does and why it matters. Include enough context that a fresh Claude Code session can understand the purpose without reading other chunks.>

## Implementation Steps

### 1. <Step Group Name>

- [ ] <Concrete action with file paths, function names, or shell commands>
- [ ] <Next action>

### 2. <Step Group Name>

- [ ] <Action>
- [ ] ...

## Verification

- [ ] <Build command passes>
- [ ] <Test command passes>
- [ ] <Manual check or assertion>
```

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

1. If this chunk involves writing or modifying code, invoke the code skill first: `skill: "code"` (no args). Follow its coding preferences and TDD workflow guidance throughout execution.
2. Read the chunk file
3. Check the "Depends on" line. If it names a dependency, read that chunk file and verify all its checkboxes are checked. If any are unchecked, STOP and report the dependency is incomplete.
4. Scan for the first unchecked `- [ ]` item. If all items are checked, report "Chunk already complete" and stop.
5. Starting from that item, execute each unchecked step in order:
   a. Perform the action described
   b. Mark the checkbox: replace `- [ ]` with `- [x]` in the chunk file
   c. After completing each numbered step group, run the build command and fix any errors before moving on
6. After all Implementation Steps are checked, execute the Verification section the same way
7. When all checkboxes are checked, report "Chunk complete"

## Rules

- If a step fails and you cannot fix it within 3 attempts, STOP and report the failure with details
- MUST mark each checkbox immediately after completing it -- this is the progress ledger
- MUST NOT skip ahead or reorder steps
- MUST run the build command after each step group, not just at the end
- For TDD chunks (step groups named "Red/Green/Refactor"), MUST verify test failures before implementing and test passes after implementing

---
```

## Chunk Writer Subagent Prompt Template

This prompt is used by the plan-task operation to delegate chunk file writing to a Task subagent. Fill in all `<...>` placeholders.

```markdown
Write a plan chunk file to <output-file-path>.

## Chunk Details

- **Number**: <NN>
- **Title**: <chunk title>
- **Depends on**: <chunk-NN-slug.md, or "None">
- **Summary**: <2-4 sentences explaining what this chunk does and why>

## High-Level Steps

<Numbered list of the high-level implementation steps from the decomposition, with enough detail for the subagent to expand into checkboxes>

## Codebase Context

<Relevant file paths, function/type names, patterns, and conventions the chunk will touch. Include enough detail that a fresh session can write precise checkboxes without exploring the codebase.>

## Build and Test

- Build: `<build command>`
- Test: `<test command>`

## Output Format

Write the chunk file using this exact template structure:

# Chunk <NN>: <Title>

**Depends on**: <dependency>

## What and Why

<2-4 sentences with enough context for a fresh Claude Code session to understand the purpose without reading other chunks.>

## Implementation Steps

### 1. <Step Group Name>

- [ ] <Concrete action with file paths, function names, or shell commands>
- [ ] <Next action>

### 2. <Step Group Name>

- [ ] <Action>
- [ ] ...

## Verification

- [ ] <Build command passes>
- [ ] <Test command passes>
- [ ] <Manual check or assertion>

## Rules

- Target ~15-25 total checkboxes. If it would exceed 25, report that the chunk should be split.
- Each checkbox MUST be completable in a single focused action.
- Use specific file paths, function names, and shell commands -- not vague descriptions.
- The "What and Why" section MUST be self-contained: a fresh Claude Code session should understand the chunk without reading other chunks or having prior conversation context.
- For chunks adding testable behavior, MUST use TDD structure: "Red" step group (write failing tests), "Green" step group (implement to pass), "Refactor" step group (clean up). Include explicit "run tests -- confirm fail/pass" checkboxes.
- For pure refactoring, config, or glue code, use plain step groups without TDD structure.
```

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
