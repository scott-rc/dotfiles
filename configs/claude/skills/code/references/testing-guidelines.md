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

**File-driven tests** — best when inputs or outputs are large, multi-line, or binary. Place paired `*.input`/`*.golden` files in a `testdata/` directory; the test globs and compares. Adding a case means adding a file pair.

**Not everything fits in a table.** If cases need different setup or assertions, write separate test functions.

## Separate Test Data from Test Logic

- Adding a new test case should never require touching assertion logic — keep data (case lists, fixture files) separate from the test body.

## Design for Testability

- Extract pure logic from I/O boundaries into input/output functions. If testing requires mocking the terminal, network, or filesystem, the logic is too coupled to I/O.

## Test the Real Code Path

Tests MUST call the actual function under test, not simulate what it does by manually setting state. A test that sets `state.cursor = 5` then asserts `state.cursor == 5` bypasses the real code path -- call `handle_key(state, key)` and check the result instead. Bypass tests give false confidence.

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

Failure messages MUST answer: what input, what was expected, what actually happened. Bad: `assertion failed`, `expected true`. Good: `count("café"): expected 4, received 5`. For complex outputs, use diffs.

## Snapshot Testing

Use snapshot testing when expected output is large or changes with the code. Use the language's idiomatic library:

- **TypeScript/JavaScript**: Vitest/Jest `toMatchInlineSnapshot()` (small), `toMatchSnapshot()` (large)
- **Go**: `gotest.tools/v3/golden` — update with `-test.update-golden`
- **Rust**: `insta` crate — `assert_snapshot!`, update with `cargo insta review`

Prefer inline snapshots for short values so expected output stays visible in the test.

## Compare Against Reference Implementations

When a reference implementation exists (naive version, standard library, external tool), use it as an oracle — run both on the same input and compare. Also validate against published test vectors or specification examples. This catches subtle bugs that hand-written expected values miss.

## Test Infrastructure Is Code

- Write helpers, custom matchers, and fixture builders to reduce per-test boilerplate. Mark helpers so failures report the caller's location. Refactor tests when they're hard to read or extend.
