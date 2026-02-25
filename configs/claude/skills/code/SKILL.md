---
name: code
description: Writes, reviews, tests, and optimizes code -- enforces TDD for new features and bug fixes, runs code review, benchmarks, and mutation testing.
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
Decompose a coding task into ordered chunks with TDD structure for testable behavior.
See [plan.md](plan.md) for detailed instructions.

### Review
Evaluate code for test gaps, idiomaticity, simplification opportunities, and correctness issues. Automatically decomposes large scopes (>8 files or >500 lines) into parallel review subagents for thorough analysis.
See [review.md](review.md) for detailed instructions.

### Benchmark (BDD)
Write a benchmark that captures the performance target, then write or optimize code to meet it.
See [benchmark.md](benchmark.md) for detailed instructions.

### Mutation Testing
Evaluate test suite quality by mutating source code and checking whether tests catch the changes, then write tests to kill survivors.
See [mutate.md](mutate.md) for detailed instructions.

## Combined Operations

Users often request multiple operations together. Handle these as follows:

**Default: TDD for new behavior.** When the request adds new functionality or fixes a bug, use the TDD operation — don't wait for the user to say "TDD" or "write tests first".

- **plan / design / "how should I implement"** → MUST invoke compose: `skill: "compose", args: "plan this task"`. See Plan operation above.
- **implement / add feature / write** (new behavior) → New Feature
- **fix bug / debug** → Fix Bug
- **refactor / clean up / rename / restructure** → Apply
- **write code** (ambiguous) → New Feature if behavior; Apply if config/glue/scripting
- **review / code review / check for issues** → Review
- **thorough / deep review** → Review (forces subagent decomposition)
- **backfill tests / add coverage / write tests** (existing code) → Add Coverage
- **benchmark / optimize** (with perf target) → Benchmark
- **optimize** (no target, general cleanup) → Apply
- **review and fix** → Review, then Apply or Fix Bug. Thorough path offers fix plan at step 13.
- **fix then add coverage** → Fix Bug, then Add Coverage
- **implement and benchmark** → New Feature, then Benchmark
- **mutate / mutation test / test my tests** → Mutation Testing
- **add coverage then mutate** → Add Coverage, then Mutation Testing
- **skip tests / no tests / just the code** → Apply (user opts out of TDD)

**When to use Apply instead of TDD**: Refactoring already-tested code, config file changes, shell scripts, glue code, one-line fixes where a test would be pure overhead, or when the user explicitly opts out.

## References

- [general-guidelines.md](general-guidelines.md) — Language-agnostic naming, comments, error handling, control flow, abstractions, testing, and string conventions
- [testing-guidelines.md](testing-guidelines.md) — Test design patterns: case structure, data separation, exhaustiveness, special cases, failure readability, golden files, and test infrastructure
- [typescript-guidelines.md](typescript-guidelines.md) — TypeScript-specific type annotations, function style, and imports
- [go-guidelines.md](go-guidelines.md) — Go-specific naming, error handling, interfaces, structs, concurrency, testing, and logging conventions
- [rust-guidelines.md](rust-guidelines.md) — Rust-specific error handling, type patterns, module organization, testing, and style conventions
- [shell-guidelines.md](shell-guidelines.md) — Bash and Fish conventions, shellcheck enforcement
- [test-environment.md](test-environment.md) — Test and benchmark runner detection, file placement conventions, and framework setup
- [review-template.md](review-template.md) — Subagent prompt template for code review tasks
- [review-checklist.md](review-checklist.md) — Five-category review checklist (test coverage, idiomaticity, simplification, correctness, naming)
- [load-guidelines.md](load-guidelines.md) — Shared guideline loading checklist referenced by all TDD/coverage/benchmark/mutation operations
