# Rust Guidelines

## Error Handling

No custom error types, `Result` types, or error enums. Fatal errors use `eprintln!()` + `process::exit(1)`. Recoverable errors use `unwrap_or()` or `unwrap_or_else()` with safe defaults. Error messages MUST include context:

```rust
let repo = git::repo_root(&cwd).unwrap_or_else(|| {
    eprintln!("gd: not a git repository");
    std::process::exit(1);
});
```

## Types

- Newtype indices for bounds-checked access (`FileIx(usize)`, `LineIx(usize)`) with `Option`-returning constructors
- Flat public-field structs for data containers
- Enums for tagged state and action results (`KeyResult`, `ViewScope`)
- `LazyLock` for compiled regexes and one-time-initialized values
- `Rc<str>` for zero-copy path sharing in performance-critical paths
- Derive `Debug, Clone, Copy, PartialEq, Eq` where applicable; `Debug` always present

## Module Organization

- `pub mod` declarations in `lib.rs` or `main.rs`; private submodules for internal structure
- Re-export shared utilities at the crate root
- `pub(crate)` for internal-only types
- ANSI utilities centralized in `tui::ansi` -- consumer crates MUST reuse, not duplicate

## Functions

- Named functions for algorithms, closures for callbacks
- Iterator chains for transformations
- `&mut` references for in-place operations
- `Option`/`Result` combinators (`map`, `and_then`, `unwrap_or`) over `match` for simple cases

## Imports

- Explicit paths, no wildcard imports in production code
- Group related imports with nested `use`:

```rust
use tui::highlight::{HighlightLines, SYNTAX_SET, THEME, highlight_line};
```

## Testing

- Unit tests in colocated `#[cfg(test)] mod tests` or `tests/` subdirectories within `src/`
- Integration tests spawn the binary via `env!("CARGO_BIN_EXE_<name>")` and check stdout/stderr/exit codes
- Fixture-driven rendering tests: `.md` + `.expected.txt` pairs registered via macros
- Shared test helpers in `tests/common.rs` with `#[allow(dead_code)]` where needed
- `Vec::with_capacity()` for pre-allocated collections in benchmarks and hot paths

## Style

- Constants at module top for color palettes and config values (`const HEADING_BLUE: u32 = 0x79c0ff;`)
- `format!()` for string building; `write!()` for efficient accumulation
- Comments only where intent is non-obvious -- no restating the code
- `debug_assert!()` for invariant checks in performance-sensitive paths
