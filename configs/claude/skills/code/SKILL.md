---
name: code
description: Loads coding style preferences and guides test-driven and benchmark-driven development workflows when writing, modifying, reviewing, planning, or optimizing code. Use when the user asks to write code, implement a feature, plan or design code, refactor, fix a bug, write tests, review code, review this code, code review, check for issues, audit code, TDD a feature, write tests first, add a regression test, fix a bug with tests, backfill tests, add coverage, write characterization tests, benchmark code, optimize performance, BDD a feature, or write benchmarks first.
---

# Code

Load and apply the user's coding style preferences when producing or reviewing code. Default to test-driven development — write tests first for new features and bug fixes. Use Apply only for changes that don't warrant tests (refactoring under existing coverage, config, glue code, one-liners).

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

### Plan
Design an implementation plan, structuring around TDD when the task involves testable behavior.
See [plan.md](plan.md) for detailed instructions.

### Review
Evaluate code for test gaps, idiomaticity, simplification opportunities, and correctness issues.
See [review.md](review.md) for detailed instructions.

### Benchmark (BDD)
Write a benchmark that captures the performance target, then write or optimize code to meet it.
See [benchmark.md](benchmark.md) for detailed instructions.

## Combined Operations

Users often request multiple operations together. Handle these as follows:

**Default: TDD for new behavior.** When the request adds new functionality or fixes a bug, use the TDD operation — don't wait for the user to say "TDD" or "write tests first".

- **"plan this"** / **"design this"** / **"how should I implement this"** (enters plan mode) → Plan
- **"implement this"** / **"add this feature"** / **"write this"** (new functionality) → New Feature
- **"fix this bug"** / **"debug this"** → Fix Bug
- **"refactor"** / **"clean up"** / **"rename"** / **"restructure"** → Apply (existing tests cover it)
- **"write code"** (ambiguous) → New Feature if it involves behavior; Apply if it's config/glue/scripting
- **"review code"** / **"review this"** / **"code review"** / **"check for issues"** / **"audit this code"** → Review
- **"backfill tests"** / **"add coverage"** / **"characterize this code"** / **"write tests"** (for existing code) → Add Coverage
- **"benchmark this"** / **"BDD this"** / **"write a benchmark first"** → Benchmark
- **"optimize this"** (with a specific performance target) → Benchmark
- **"optimize this"** (general cleanup, no performance target) → Apply
- **"review and fix"** / **"review then fix the issues"** → Review, then Apply (or Fix Bug if a specific bug is found)
- **"fix and add coverage for the rest"** → Fix Bug, then Add Coverage
- **"implement and benchmark"** → New Feature, then Benchmark
- **"skip tests"** / **"no tests"** / **"just the code"** → Apply (user explicitly opts out of TDD)

**When to use Apply instead of TDD**: Refactoring already-tested code, config file changes, shell scripts, glue code, one-line fixes where a test would be pure overhead, or when the user explicitly opts out.

**Important**: You MUST read and follow the detailed instruction file for each operation before executing it. Do not rely on the summaries above.

## References

- [general-guidelines.md](general-guidelines.md) — Language-agnostic naming, comments, error handling, control flow, abstractions, testing, and string conventions
- [typescript-guidelines.md](typescript-guidelines.md) — TypeScript-specific type annotations, function style, and imports
- [go-guidelines.md](go-guidelines.md) — Go-specific naming, error handling, interfaces, structs, concurrency, testing, and logging conventions
- [shell-guidelines.md](shell-guidelines.md) — Bash and Fish conventions, shellcheck enforcement
- [test-environment.md](test-environment.md) — Test and benchmark runner detection, file placement conventions, and framework setup
