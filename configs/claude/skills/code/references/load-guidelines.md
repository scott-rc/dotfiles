# Coding Guidelines Index

Reference files for coding and testing conventions, loaded by TDD, coverage, benchmark, and mutation operations.

## Core Guidelines (all operations)

- `general-guidelines.md` — language-agnostic naming, comments, error handling, control flow, abstractions, and string conventions
- `testing-guidelines.md` — test design patterns: parameterization, data separation, exhaustiveness, special cases, failure readability, and snapshot testing
- `test-environment.md` — test and benchmark runner detection, file placement conventions, and framework setup

## Language-Specific Guidelines (load if applicable)

- `typescript-guidelines.md` — TypeScript type annotations, function style, and imports
- `go-guidelines.md` — Go naming, error handling, interfaces, structs, concurrency, and testing
- `rust-guidelines.md` — Rust error handling, type patterns, module organization, and testing
- `shell-guidelines.md` — Bash and Fish conventions, shellcheck enforcement

For languages without a dedicated guideline file (Python, Java, Ruby, etc.), apply general-guidelines.md and infer conventions from the project's existing code.
