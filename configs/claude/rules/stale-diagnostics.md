# Stale Language Server Diagnostics

Diagnostics shown after file edits often reflect an intermediate state — the language server lags behind and may report issues that no longer exist.

## Rules

- Complete all planned edits before evaluating any diagnostics. Only those that persist afterward warrant action.
- Before acting on a diagnostic, re-read the file at the referenced line and confirm the issue still exists. If the line was recently edited, treat the diagnostic as stale until confirmed.
- Diagnostics in files that import from edited files can also be stale — apply the same check before acting.

## Example

These patterns appear across all language servers (illustrative, not exhaustive):
- TypeScript: `'X' is not defined`, `'X' is declared but never used`
- Rust: `cannot borrow 'X' as mutable`, `use of moved value`
- Go: `undefined: X`, `X declared and not used`

Any such diagnostic on a line you just changed — re-read the file first; do not attempt a fix without confirming.
