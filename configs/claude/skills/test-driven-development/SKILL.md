---
name: test-driven-development
description: Guides test-driven development workflows for any language. Use when the user asks to TDD a feature, write tests first, add a regression test, fix a bug with tests, backfill tests, add coverage, or write characterization tests.
---

# Test-Driven Development

Drive implementation through tests — write the test first, watch it fail, make it pass, then refactor.

## Operations

### New Feature
Full red-green-refactor cycle for building a feature test-first.
See [new-feature.md](new-feature.md) for detailed instructions.

### Fix Bug
Write a regression test that captures the bug, then fix it.
See [fix-bug.md](fix-bug.md) for detailed instructions.

### Add Coverage
Write characterization tests for existing untested code.
See [add-coverage.md](add-coverage.md) for detailed instructions.

## Combined Operations

Users often request multiple operations together. Handle these as follows:

- **"TDD this feature"** / **"write tests first"** → New Feature
- **"test and fix this bug"** / **"regression test"** → Fix Bug
- **"backfill tests"** / **"add coverage"** / **"characterize this code"** → Add Coverage
- **"TDD the fix and add coverage for the rest"** → Fix Bug, then Add Coverage

**Important**: You MUST read and follow the detailed instruction file for each operation before executing it. Do not rely on the summaries above.

## References

- [test-environment.md](test-environment.md) — Test runner detection, file placement conventions, and framework setup
