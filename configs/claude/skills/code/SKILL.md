---
name: code
description: Writes, reviews, tests, and optimizes code, and designs architectural refactors -- enforces TDD for new features and bug fixes, runs code review, benchmarks, mutation testing, and deep-module refactor RFCs.
argument-hint: "[write | test | review | benchmark | architect] [context]"
---

# Code

Load and apply the user's coding style preferences when producing or reviewing code. Default to test-driven development — write tests first for new features and bug fixes. Use Write (Apply mode) only for changes that don't warrant tests (refactoring under existing coverage, config, glue code, one-liners).

## Operations

### Write
Load preferences and write code — **Apply mode** (no TDD) for refactoring, config, and glue; **TDD mode** (red-green-refactor) for any behavior change, including new features and bug fixes.
MUST read operations/write.md before executing.

### Test
Improve or evaluate test coverage — Coverage mode (characterization tests for untested code) or Mutate mode (mutation testing to find and kill survivors).
MUST read operations/test.md before executing.

### Review
Evaluate code for test gaps, idiomaticity, simplification opportunities, architectural smells, and correctness issues. Automatically decomposes large scopes (>8 files or >500 lines) into parallel review subagents for thorough analysis. Supports an evaluate-fix loop mode for iterative convergence.
MUST read operations/review.md before executing.

### Benchmark
Write a benchmark that captures the performance target, then write or optimize code to meet it.
MUST read operations/benchmark.md before executing.

### Architect
Design an architectural refactor. Identifies shallow modules or friction points (discovery mode) or takes a specified target, frames the problem, dispatches 4 parallel design subagents with different constraints, and outputs a refactor RFC at `./tmp/refactor/<name>/rfc.md`. Does NOT implement — the RFC is terminal output; hand off to `plan create` for implementation phasing or `git` to push as an issue.
MUST read operations/architect.md before executing.

## Combined Operations

Users often request multiple operations together. Handle these as follows:

**Default: TDD for any behavior change.** When the request adds new functionality or fixes a bug, use Write (TDD mode) — don't wait for the user to say "TDD" or "write tests first".

- **plan / design / "how should I implement"** → `plan create` (plan skill)
- **"continue implementing" / "implement plan" / "execute plan" / "run plan"** (with a plan file path) → `plan execute` (plan skill)
- **add feature / write** (new behavior) → Write (TDD mode)
- **implement / full cycle / build and verify** → Write (TDD mode), then Review (loop mode) on all changed files
- **implement with mutation testing / harden** → Write (TDD mode), then Test (Mutate mode), then Review (loop mode) on all changed files
- **fix bug / debug** → Write (TDD mode, bug-fix sub-case)
- **refactor / clean up / rename / restructure** (small scope) → Write (Apply mode)
- **make easier to maintain** (no specific target) → Architect (discovery mode)
- **deepen module / simplify interface / hide complexity** → Architect
- **architect / architectural review / find refactor candidates** → Architect (discovery mode)
- **refactor RFC / design a refactor / propose refactor** → Architect
- **module boundaries / ports and adapters / dependency strategy** → Architect
- **large refactor / big refactor / restructure the X** → Architect (targeted on X)
- **ensure / make X do Y / add support for / sync / integrate** → Write (TDD mode) — these add behavior even when they modify existing files
- **write code** (ambiguous) → Write (TDD mode) if it changes runtime behavior; Write (Apply mode) if config/glue/scripting
- **review / code review / check for issues** → Review
- **thorough / deep review** → Review (forces subagent decomposition)
- **backfill tests / add coverage / write tests** (existing code) → Test (Coverage mode)
- **benchmark / optimize** (with perf target) → Benchmark
- **optimize** (no target, general cleanup) → Write (Apply mode)
- **review and fix** → Review, then Write (Apply mode for style/convention changes; TDD mode, bug-fix sub-case, for behavioral issues). Non-loop mode: thorough path offers a fix plan after reporting findings. When chaining to TDD bug-fix, write the regression test before fixing — do not skip this precondition.
- **review and loop / review and fix loop** → Review (loop mode — drives evaluate-fix cycle until convergence)
- **fix then add coverage** → Write (TDD mode, bug-fix sub-case), then Test (Coverage mode)
- **implement and benchmark** → Write (TDD mode), then Benchmark
- **mutate / mutation test / test my tests** → Test (Mutate mode)
- **add coverage then mutate** → Test (Coverage mode), then Test (Mutate mode)
- **skip tests / no tests / just the code** → Write (Apply mode — user opts out of TDD)
- **design and implement** (for a refactor) → Architect, then user reviews RFC, then `plan create` on the RFC, then `plan execute` on the plan

**When to use Write (Apply mode) instead of TDD**: refactoring already-tested code, config file changes, shell scripts, glue code, one-line fixes where a test would be pure overhead, or when the user explicitly opts out.

**When to use Architect instead of Write (Apply mode)**: the interface itself is the question; scope is large enough to warrant a design document first; shallow modules or cross-cutting coupling that a mechanical refactor can't address.

## Delegation

Global delegation rules apply. Code-skill-specific additions:

- **Mode selection stays inline** — the orchestrator chooses Apply/TDD before delegating. Reading file lists and checking test infrastructure to inform that choice is fine.
- **Broad refactors get a scope first** — for vague maintainability requests, identify the primary seam, expected touched files, and done condition before editing or delegating. If the scope is architectural (shallow modules, boundary redesign), route to Architect instead of Write.
- **Call out repo state early** — if unrelated files are already modified, say so and limit edits to the intended scope.
- **Simple changes stay inline** — single-file edits, config changes, and small fixes don't need a subagent. Delegate when the task spans multiple files or benefits from TDD scaffolding.
- **Describe behavior, not code changes** — when delegating, tell the subagent what to add/fix/test and where the code lives. Do NOT prescribe struct fields, function signatures, or file-by-file edit lists.
- **Architect uses 4 parallel design subagents** — the orchestrator picks subagent type at dispatch; passes the framing + one of the 4 design constraints per agent; collates results for user selection.
- **Validate subagent results** — if the result summary is empty or the subagent ran under 30 seconds on a non-trivial task, investigate before reporting success.

## References

- references/general-guidelines.md — Language-agnostic naming, comments, error handling, control flow, abstractions, and string conventions
- references/testing-guidelines.md — Test design philosophy (behavior over implementation) plus patterns: case structure, data separation, exhaustiveness, special cases, failure readability, golden files, and test infrastructure
- references/test-examples.md — Good vs bad test examples (language-agnostic prose with TypeScript illustrations)
- references/mocking.md — When to mock (boundaries only) and how to design mockable boundaries (DI, SDK-style adapters)
- references/interface-design.md — Testability principles: accept dependencies, functional returns, minimal interface complexity
- references/deep-modules.md — Ousterhout summary: small interface + lots of implementation; signals of shallowness
- references/refactor-smells.md — Local smells (duplication, long methods, feature envy, primitive obsession) and structural smells (shallow modules, tight clusters) that point to refactor or Architect
- references/dependency-categories.md — In-process / local-substitutable / ports-and-adapters / mock-external — determines deepening strategy and testing approach
- references/typescript-guidelines.md — TypeScript-specific type annotations, function style, and imports
- references/go-guidelines.md — Go-specific naming, error handling, interfaces, structs, concurrency, testing, and logging conventions
- references/rust-guidelines.md — Rust-specific error handling, type patterns, module organization, testing, and style conventions
- references/shell-guidelines.md — Bash and Fish conventions, shellcheck enforcement
- references/test-environment.md — Test and benchmark runner detection, file placement conventions, and framework setup
- references/load-guidelines.md — Index of all coding guideline files with descriptions, referenced by all operations that load language-specific guidelines
- references/review-checklist.md — Review criteria for test coverage, idiomaticity, simplification, architectural smells, correctness, and naming, referenced by review subagents
