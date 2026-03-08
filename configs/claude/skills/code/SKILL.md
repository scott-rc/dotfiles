---
name: code
description: Writes, reviews, tests, and optimizes code -- enforces TDD for new features and bug fixes, runs code review, benchmarks, and mutation testing.
argument-hint: "[write | test | plan | review | benchmark | implement] [context]"
---

# Code

Load and apply the user's coding style preferences when producing or reviewing code. Default to test-driven development — write tests first for new features and bug fixes. Use Write (Apply mode) only for changes that don't warrant tests (refactoring under existing coverage, config, glue code, one-liners).

## Operations

### Write
Load preferences and write code — Apply mode (no TDD) for refactoring, config, and glue; Feature mode (red-green-refactor) for new behavior; Fix mode (regression-first) for bug fixes.
See operations/write.md for detailed instructions.

### Test
Improve or evaluate test coverage — Coverage mode (characterization tests for untested code) or Mutate mode (mutation testing to find and kill survivors).
See operations/test.md for detailed instructions.

### Plan
Decompose a coding task into ordered chunks with TDD structure for testable behavior.
See operations/plan.md for detailed instructions.

### Review
Evaluate code for test gaps, idiomaticity, simplification opportunities, and correctness issues. Automatically decomposes large scopes (>8 files or >500 lines) into parallel review subagents for thorough analysis. Supports an evaluate-fix loop mode for iterative convergence.
See operations/review.md for detailed instructions.

### Benchmark
Write a benchmark that captures the performance target, then write or optimize code to meet it.
See operations/benchmark.md for detailed instructions.

### Implement
Build with TDD, harden with mutation testing, and review-fix until converged — end-to-end verification loop.
See operations/implement.md for detailed instructions.

## Combined Operations

Users often request multiple operations together. Handle these as follows:

**Default: TDD for new behavior.** When the request adds new functionality or fixes a bug, use the TDD operation — don't wait for the user to say "TDD" or "write tests first".

- **plan / design / "how should I implement"** → Plan (see operations/plan.md)
- **add feature / write** (new behavior) → Write (Feature mode)
- **implement and review / full cycle / build and verify / implement with mutation testing** → Implement
- **fix bug / debug** → Write (Fix mode)
- **refactor / clean up / rename / restructure** → Write (Apply mode)
- **write code** (ambiguous) → Write (Feature mode) if behavior; Write (Apply mode) if config/glue/scripting
- **review / code review / check for issues** → Review
- **thorough / deep review** → Review (forces subagent decomposition)
- **backfill tests / add coverage / write tests** (existing code) → Test (Coverage mode)
- **benchmark / optimize** (with perf target) → Benchmark
- **optimize** (no target, general cleanup) → Write (Apply mode)
- **review and fix** → Review, then Write (Apply mode for style/convention changes; Fix mode for behavioral issues). Non-loop mode: thorough path offers a fix plan after reporting findings. When chaining to Fix mode, write the regression test before fixing — do not skip this precondition.
- **review and loop / review and fix loop** → Review (loop mode — drives evaluate-fix cycle until convergence)
- **fix then add coverage** → Write (Fix mode), then Test (Coverage mode)
- **implement and benchmark** → Write (Feature mode), then Benchmark
- **mutate / mutation test / test my tests** → Test (Mutate mode)
- **add coverage then mutate** → Test (Coverage mode), then Test (Mutate mode)
- **skip tests / no tests / just the code** → Write (Apply mode — user opts out of TDD)

**When to use Write (Apply mode) instead of TDD**: Refactoring already-tested code, config file changes, shell scripts, glue code, one-line fixes where a test would be pure overhead, or when the user explicitly opts out.

## References

- references/general-guidelines.md — Language-agnostic naming, comments, error handling, control flow, abstractions, and string conventions
- references/testing-guidelines.md — Test design patterns: case structure, data separation, exhaustiveness, special cases, failure readability, golden files, and test infrastructure
- references/typescript-guidelines.md — TypeScript-specific type annotations, function style, and imports
- references/go-guidelines.md — Go-specific naming, error handling, interfaces, structs, concurrency, testing, and logging conventions
- references/rust-guidelines.md — Rust-specific error handling, type patterns, module organization, testing, and style conventions
- references/shell-guidelines.md — Bash and Fish conventions, shellcheck enforcement
- references/test-environment.md — Test and benchmark runner detection, file placement conventions, and framework setup
- references/load-guidelines.md — Index of all coding guideline files with descriptions, referenced by all operations that load language-specific guidelines
- references/review-checklist.md — Review criteria for test coverage, idiomaticity, simplification, correctness, and naming, referenced by review subagents
