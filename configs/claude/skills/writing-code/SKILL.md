---
name: writing-code
description: Loads coding style preferences and guides test-driven development workflows when writing, modifying, or reviewing code. Use when the user asks to write code, implement a feature, refactor, fix a bug, write tests, review code, TDD a feature, write tests first, add a regression test, fix a bug with tests, backfill tests, add coverage, or write characterization tests.
---

# Writing Code

Load and apply the user's coding style preferences when producing or reviewing code. When the workflow is test-driven, follow the appropriate TDD operation.

## Operations

### Apply
Load preferences, write code, and verify output matches the guidelines.
See [apply.md](apply.md) for detailed instructions.

### New Feature (TDD)
Full red-green-refactor cycle for building a feature test-first.
See [new-feature.md](new-feature.md) for detailed instructions.

### Fix Bug (TDD)
Write a regression test that captures the bug, then fix it.
See [fix-bug.md](fix-bug.md) for detailed instructions.

### Add Coverage
Write characterization tests for existing untested code.
See [add-coverage.md](add-coverage.md) for detailed instructions.

## Combined Operations

Users often request multiple operations together. Handle these as follows:

- **"write code"** / **"implement this"** / **"refactor"** / **"review code"** → Apply
- **"TDD this feature"** / **"write tests first"** → New Feature
- **"fix this bug"** (no mention of tests) → Apply
- **"test and fix this bug"** / **"regression test"** / **"fix a bug with tests"** → Fix Bug
- **"backfill tests"** / **"add coverage"** / **"characterize this code"** / **"write tests"** (for existing code) → Add Coverage
- **"TDD the fix and add coverage for the rest"** → Fix Bug, then Add Coverage
- **"implement this feature with tests"** → New Feature

**Important**: You MUST read and follow the detailed instruction file for each operation before executing it. Do not rely on the summaries above.

All TDD operations (New Feature, Fix Bug, Add Coverage) MUST also load and apply the coding style guidelines when writing code and tests.

## References

- [general-guidelines.md](general-guidelines.md) — Language-agnostic naming, comments, error handling, control flow, abstractions, testing, and string conventions
- [typescript-guidelines.md](typescript-guidelines.md) — TypeScript-specific type annotations, function style, and imports
- [test-environment.md](test-environment.md) — Test runner detection, file placement conventions, and framework setup
