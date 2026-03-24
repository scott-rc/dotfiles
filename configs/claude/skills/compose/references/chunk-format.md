# Chunk File Format

Canonical format for plan chunk files. Both TDD and non-TDD structures share the same skeleton.

## Structure

```markdown
# Chunk <NN>: <Title>

**Depends on**: <chunk-NN-slug.md, or "None">

## What and Why

<2-4 sentences. MUST be self-contained: a fresh session understands the purpose without reading other chunks.>

## Implementation Steps

<step groups -- see mode-specific sections below>

## Verification

- [ ] <Build command passes>
- [ ] <Test command passes>
- [ ] <Manual check or assertion>
```

## TDD Step Groups

Use when steps mention writing tests, adding testable behavior, or implementing features with test coverage.

```markdown
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
```

## Non-TDD Step Groups

Use for pure refactoring, config changes, glue code, docs.

```markdown
### 1. <Step Group Name>

- [ ] <Concrete action with file paths, function names, or shell commands>
- [ ] <Next action>

### 2. <Step Group Name>

- [ ] <Action>
```

## Rules

- MUST use specific file paths, function names, and shell commands -- not vague descriptions
- Each checkbox MUST be completable in a single focused action
- "What and Why" MUST be self-contained: no references to other chunks or prior conversation
- TDD step groups MUST include explicit "run tests -- confirm fail/pass" checkboxes
- Target 15-25 total checkboxes. If count exceeds 25, the chunk needs splitting
- If a chunk touches more than 15 files, it needs splitting -- even if checkbox count is under 25
- Verification section MUST include at least one build and one test checkbox
