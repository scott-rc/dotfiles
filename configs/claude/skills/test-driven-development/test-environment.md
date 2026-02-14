# Test Environment

How to detect and configure the test environment for a project. All operations reference this file before writing tests.

## Test Runner Detection

Resolve the test runner in priority order. Stop at the first match.

1. **Project instructions**: Check CLAUDE.md or README for an explicit test command.
2. **Config file detection**: Look for framework config files in the project root:

   | Config file | Framework | Run command |
   |---|---|---|
   | `jest.config.*`, `jest` key in `package.json` | Jest | `npx jest` |
   | `vitest.config.*`, `vitest` key in `package.json` | Vitest | `npx vitest run` |
   | `playwright.config.*` | Playwright | `npx playwright test` |
   | `cypress.config.*` | Cypress | `npx cypress run` |
   | `pytest.ini`, `pyproject.toml` with `[tool.pytest]`, `setup.cfg` with `[tool:pytest]` | pytest | `pytest` |
   | `Cargo.toml` | Cargo test | `cargo test` |
   | `go.mod` | Go test | `go test ./...` |
   | `mix.exs` | ExUnit | `mix test` |
   | `Gemfile` with `rspec`, `.rspec` | RSpec | `bundle exec rspec` |
   | `build.gradle*` | Gradle | `./gradlew test` |
   | `pom.xml` | Maven | `mvn test` |
   | `deno.json*` | Deno test | `deno test` |

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

| Language | Pattern | Example |
|---|---|---|
| TypeScript/JavaScript | `*.test.{ts,js}` or `*.spec.{ts,js}` | `parser.test.ts` |
| Python | `test_*.py` | `test_parser.py` |
| Go | `*_test.go` | `parser_test.go` |
| Rust | `mod tests` in source, or `tests/*.rs` | inline or `tests/parser.rs` |
| Ruby | `*_spec.rb` or `test_*.rb` | `parser_spec.rb` |
| Elixir | `*_test.exs` | `parser_test.exs` |
| Java/Kotlin | `*Test.java` / `*Test.kt` | `ParserTest.java` |

## Framework Setup

If no test framework is detected and none can be inferred:

1. Suggest an appropriate framework for the language (prefer the most common/standard choice).
2. Get user approval before installing anything.
3. Install and configure with minimal setup — only what's needed to run tests.
