---
name: explain
description: Explains code changes — branch diffs, individual commits, specific files, or commit ranges — when the user asks to explain, understand, walk through, summarize, or review what changed.
argument-hint: "[commit | file | branch | range]"
---

# Explain Changes

Help the user understand code changes by producing layered explanations (why, what, how) adapted to diff size.

## Operations

### Explain
Parse arguments, gather context, read changed files, and produce a layered explanation.
See [explain.md](explain.md) for detailed instructions.

## References

These files are referenced by the operation instructions:

- [explain-patterns.md](explain-patterns.md) - Argument classification, diff size thresholds, large diff strategy, output format
