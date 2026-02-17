---
name: code
description: Loads coding style preferences and guides test-driven and benchmark-driven development workflows when writing, modifying, reviewing, planning, or optimizing code. Use when the user asks to write code, implement a feature, plan or design code, refactor, fix a bug, write tests, review code, TDD a feature, write tests first, add a regression test, fix a bug with tests, backfill tests, add coverage, write characterization tests, benchmark code, optimize performance, BDD a feature, or write benchmarks first.
---

# Code

Load and apply the user's coding style preferences when producing or reviewing code. When the workflow is test-driven or benchmark-driven, follow the appropriate operation.

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

### Benchmark (BDD)
Write a benchmark that captures the performance target, then write or optimize code to meet it.
See [benchmark.md](benchmark.md) for detailed instructions.

## Combined Operations

Users often request multiple operations together. Handle these as follows:

- **"write code"** / **"implement this"** / **"refactor"** / **"review code"** → Apply
- **"TDD this feature"** / **"write tests first"** → New Feature
- **"fix this bug"** (no mention of tests) → Apply
- **"test and fix this bug"** / **"regression test"** / **"fix a bug with tests"** → Fix Bug
- **"backfill tests"** / **"add coverage"** / **"characterize this code"** / **"write tests"** (for existing code) → Add Coverage
- **"benchmark this"** / **"BDD this"** / **"write a benchmark first"** → Benchmark
- **"optimize this"** (with a specific performance target) → Benchmark
- **"optimize this"** (general cleanup, no performance target) → Apply
- **"TDD the fix and add coverage for the rest"** → Fix Bug, then Add Coverage
- **"implement and benchmark"** / **"TDD then benchmark"** → New Feature, then Benchmark
- **"implement this feature with tests"** → New Feature

**Important**: You MUST read and follow the detailed instruction file for each operation before executing it. Do not rely on the summaries above.

All TDD and BDD operations (New Feature, Fix Bug, Add Coverage, Benchmark) MUST also load and apply the coding style guidelines when writing code, tests, and benchmarks.

## References

- [general-guidelines.md](general-guidelines.md) — Language-agnostic naming, comments, error handling, control flow, abstractions, testing, and string conventions
- [typescript-guidelines.md](typescript-guidelines.md) — TypeScript-specific type annotations, function style, and imports
- [test-environment.md](test-environment.md) — Test and benchmark runner detection, file placement conventions, and framework setup
