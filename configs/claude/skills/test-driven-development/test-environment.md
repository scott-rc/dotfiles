# Test Environment

How to detect and configure the test environment for a project. All operations reference this file before writing tests.

## Test Runner Detection

Resolve the test runner in priority order. Stop at the first match.

1. **Project instructions**: Check CLAUDE.md or README for an explicit test command.
2. **Config file detection**: Look for framework config files in the project root:
   - **Jest**: `jest.config.*` or `jest` key in `package.json` -- `npx jest`
   - **Vitest**: `vitest.config.*` or `vitest` key in `package.json` -- `npx vitest run`
   - **Playwright**: `playwright.config.*` -- `npx playwright test`
   - **Cypress**: `cypress.config.*` -- `npx cypress run`
   - **pytest**: `pytest.ini`, `pyproject.toml` with `[tool.pytest]`, or `setup.cfg` with `[tool:pytest]` -- `pytest`
   - **Cargo test**: `Cargo.toml` -- `cargo test`
   - **Go test**: `go.mod` -- `go test ./...`
   - **ExUnit**: `mix.exs` -- `mix test`
   - **RSpec**: `Gemfile` with `rspec` or `.rspec` -- `bundle exec rspec`
   - **Gradle**: `build.gradle*` -- `./gradlew test`
   - **Maven**: `pom.xml` -- `mvn test`
   - **Deno test**: `deno.json*` -- `deno test`

3. **Existing test files**: Infer the framework from import statements or test syntax in existing test files.
4. **Ask the user**: If nothing is detected, ask what test framework and command to use.

## Test File Placement

Detect the project's convention from existing test files:

- **Colocated**: Test files next to source files (e.g., `src/foo.ts` + `src/foo.test.ts`)
- **Separate directory**: Tests in a dedicated directory (e.g., `tests/`, `test/`, `__tests__/`, `spec/`)
- **Language convention**: Some languages have strong defaults (`_test.go` files colocated, `tests/` directory in Rust)

If no existing tests exist, ask the user where test files should go.

## Test File Naming

Follow the project's existing naming convention. Common patterns:
   - **TypeScript/JavaScript**: `*.test.{ts,js}` or `*.spec.{ts,js}` (e.g., `parser.test.ts`)
   - **Python**: `test_*.py` (e.g., `test_parser.py`)
   - **Go**: `*_test.go` (e.g., `parser_test.go`)
   - **Rust**: `mod tests` in source, or `tests/*.rs` (e.g., `tests/parser.rs`)
   - **Ruby**: `*_spec.rb` or `test_*.rb` (e.g., `parser_spec.rb`)
   - **Elixir**: `*_test.exs` (e.g., `parser_test.exs`)
   - **Java/Kotlin**: `*Test.java` / `*Test.kt` (e.g., `ParserTest.java`)

## Framework Setup

When no framework is detected: prefer the language's standard or most common framework, get user approval before installing, and configure minimally -- only what's needed to run tests.
