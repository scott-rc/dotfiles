# Testing Guidelines

Test design patterns for writing tests that are easy to extend, maintain, and debug. Apply these when writing tests in any operation.

## Make It Easy to Add New Cases

Structure tests so adding a new case requires only new data, not new logic. The test loop, assertions, and setup should be written once.

**Parameterized tests** — best when cases share identical setup/assertion logic:

```ts
const cases = [
  { name: "empty", input: "", want: 0 },
  { name: "single", input: "a", want: 1 },
  { name: "unicode", input: "caf\u00e9", want: 4 },
];

for (const { name, input, want } of cases) {
  test(name, () => {
    expect(count(input)).toBe(want);
  });
}
```

Equivalent patterns exist in every language: `test.each` (Jest/Vitest), subtests in Go, `#[test_case]` in Rust.

**File-driven tests** — best when inputs or outputs are large, multi-line, or binary. Place paired files in a fixtures directory:

```
testdata/
  simple.input     simple.golden
  nested.input     nested.golden
  unicode.input    unicode.golden
```

The test globs for `*.input`, runs the function, and compares against the corresponding `.golden` file. Adding a new case means adding a new file pair.

**Not everything fits in a table.** If cases need different setup, different assertions, or different error handling, write separate test functions. Forcing unlike cases into a table makes tests harder to read.

## Separate Test Data from Test Logic

Test data and test logic are distinct concerns. When mixed together, both are harder to change.

- Parameterized tests separate data (the case list) from logic (the test body)
- File-driven tests separate data (fixture files) from logic (the test function)
- Multi-file test archives bundle related files into a single test fixture

The goal: someone adding a new test case should never need to touch the assertion logic.

## Design for Testability

Extract pure logic from I/O boundaries (event loops, HTTP handlers, CLI main functions) into functions that take input and return output. The extracted function should be the *same* function the production code calls — not a parallel reimplementation.

Signal: if testing a behavior requires mocking the terminal, network, or filesystem, the logic is too coupled to I/O. Move the logic into a pure function, then have the I/O boundary call that function.

## Test the Real Code Path

Tests MUST call the actual function under test, not simulate what it does by manually setting state.

If a test sets fields on an object instead of calling the function that would set those fields, it's testing a mental model of the code, not the code itself. These bypass tests give false confidence — they pass even when the real code path is broken.

Example: testing a key handler by calling `handle_key(state, key)` and checking the resulting state is correct. Testing it by manually setting `state.cursor = 5` and then asserting `state.cursor == 5` is not — that test cannot catch a bug in the handler.

## Write Exhaustive Tests

When the input space is small enough, test every case. When it's not, test *categories* exhaustively:

- Every branch through the code (not just coverage — think about combinations)
- Every error path that can be triggered by external input
- Every documented behavior in the API

Coverage tools find code you forgot to test. But coverage is no substitute for thought — 100% line coverage can still miss important cases (wrong inputs, ordering issues, concurrency).

## Look for Special Cases

Before finishing a test plan, check for:

- Empty input (empty string, empty list, nil/null, zero)
- Single element
- Boundary values (off-by-one, max int, min int, exactly at limit)
- Duplicate values
- Unicode, multi-byte characters, emoji
- Negative numbers, zero, overflow
- Whitespace-only, trailing newlines, mixed line endings
- Paths with spaces, symlinks, missing directories

Not all apply to every function. But scanning this list catches cases that slip through.

## Make Test Failures Readable

When a test fails, the message should answer: what input was tested, what was expected, and what actually happened. Nobody should need to read the test source to understand a failure.

Good failure messages:

```
count("caf\u00e9"): expected 4, received 5
```

```
parseConfig("testdata/missing-field.toml"):
  expected error containing: "required field 'name'"
  received: no error
```

Bad: `assertion failed`, `expected true`, `values differ`.

For complex outputs, use diffs — most snapshot libraries show these automatically.

## Snapshot Testing

When expected output is large or changes with the code, use snapshot testing. The framework serializes actual output, compares it against a stored snapshot file, and fails on mismatch.

**Use the language's idiomatic snapshot library:**

- **TypeScript/JavaScript**: Vitest or Jest inline snapshots (`toMatchInlineSnapshot()`) for small values, file snapshots (`toMatchSnapshot()`) for larger output
- **Go**: `gotest.tools/v3/golden` — `golden.Assert` for string comparison, `golden.AssertBytes` for binary, update with `-test.update-golden`
- **Rust**: `insta` crate — `assert_snapshot!`, `assert_debug_snapshot!`, `assert_yaml_snapshot!`

**Workflow**: Run tests normally to catch regressions. Run with the update command (`--update-snapshots`, `cargo insta review`, `-test.update-golden`) to regenerate after intentional changes. Review the snapshot diff in version control.

This keeps large expected outputs out of test source and makes intentional changes a one-command operation. Prefer inline snapshots for short values (a few lines) so the expected output stays visible in the test.

## Compare Against Reference Implementations

When a reference implementation exists (a simpler but slower version, a standard library, an external tool), use it as an oracle:

- Parse the same input with both and compare outputs
- Run the same computation with a known-correct naive version
- Validate against published test vectors or specification examples

This catches subtle bugs that hand-written expected values miss.

## Test Infrastructure Is Code

Code quality is limited by test quality. Invest in test infrastructure:

- Write helpers that reduce per-test boilerplate (mark them as test helpers so failures report the caller's location, not the helper's)
- Build custom assertion functions or matchers for domain-specific comparisons
- Create fixture builders that produce valid test objects with sensible defaults
- Write parsers for test input formats when raw data is noisy
- Write printers for test output formats when raw comparison is unreadable

Improve tests over time. When a test is hard to read, refactor it. When a failure message is confusing, improve it. When adding a case is tedious, fix the test structure.
