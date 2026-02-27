# Go Guidelines

## Naming

Lean into short names. Common abbreviations: `fn`, `cfg`, `ctrl`, `tc` (test case), `req`, `resp`. Receivers are a single lowercase letter from the type (`func (c *Controller)`). Package names are single lowercase words — no underscores, no plurals.

## Error Handling

Wrap with context: `fmt.Errorf("doing thing: %w", err)`. Define sentinel errors with `errors.New` for validation failures. Check error types with `errors.Is`/`errors.As` or domain predicates (`apierrors.IsNotFound`). Reserve `panic` for programmer errors only — recover in critical loops with `defer`.

## Interfaces

Small (1-4 methods), defined near the call site, not in the implementing package. Verify compliance at package level:

```go
var _ slog.LogValuer = (*Function)(nil)
```

## Structs

Group fields by concern: config/immutable, lifecycle/context, sync primitives, owned resources, dependency injection, caches, external clients. Use functional options for constructors with optional configuration:

```go
type Option func(*Thing)

func WithTimeout(d time.Duration) Option {
	return func(t *Thing) { t.timeout = d }
}
```

## Concurrency

Prefer `atomic.Pointer[T]` with CAS for lock-free shared state over mutexes. Use typed concurrent maps over raw `sync.Map`. Context-controlled goroutines with error channels for coordination. `sync.Pool` for hot-path buffer reuse.

## Testing

- `gotest.tools/v3/assert` — `assert.NilError`, `assert.Equal`, `assert.ErrorContains`, `assert.DeepEqual`
- `t.Parallel()` on test functions and subtests by default
- `t.Helper()` on every test helper
- Shared test data in a `fixture` package with builder functions
- Poll-based assertions for eventually-consistent behavior (`poll.WaitOn`)

## Comments

Godoc format — start with the declared name. Package comments on the primary file. Inline comments explain "why" not "what":

```go
// Controller manages pod lifecycle and scaling decisions.
type Controller struct { ... }
```

## Logging

Structured logging via `slog`. Implement `slog.LogValuer` on domain types. Use a typed key package for cross-cutting concerns (logging, tracing, HTTP headers) so names stay consistent across subsystems.
