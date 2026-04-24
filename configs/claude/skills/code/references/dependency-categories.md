# Dependency Categories

When assessing a module for deepening (TDD refactor step or `code architect` design), classify each dependency. The category determines the deepening strategy and the testing approach.

## 1. In-process

Pure computation, in-memory state, no I/O. No network, no disk, no system calls.

- **Deepening strategy**: Always deepenable. Merge the shallow modules; test directly through the combined interface.
- **Testing**: Unit tests at the deepened interface. No fakes or mocks required.

## 2. Local-substitutable

Has an external dependency, but a local test stand-in exists and runs in the test suite.

Examples: PGLite for Postgres, in-memory filesystem, an in-memory cache instead of Redis.

- **Deepening strategy**: Deepenable if the substitute is viable for the behaviors under test. The deepened module owns the logic; tests run against the stand-in; production uses the real dependency.
- **Testing**: Integration tests through the public interface, with the stand-in running. Tests exercise real code paths end-to-end.

## 3. Remote but owned (Ports & Adapters)

Your own services across a network boundary -- internal APIs, microservices, message queues you operate.

- **Deepening strategy**: Define a **port** (interface) at the seam. The deepened module owns the logic; the transport is injected via the port. Production gets an HTTP/gRPC/queue adapter; tests get an in-memory adapter.
- **Testing**: Tests at the seam use the in-memory adapter. They exercise the module as one deep unit, even though the system is deployed across a network.
- **Brief phrasing**: "Define a shared interface (port), implement an HTTP adapter for production and an in-memory adapter for testing, so the logic can be tested as one deep module even though it's deployed across a network boundary."

## 4. True external (Mock-boundary)

Third-party services you don't control -- Stripe, Twilio, SaaS APIs.

- **Deepening strategy**: Mock at the seam. The deepened module takes the external dependency as an injected port (same pattern as ports-and-adapters); tests provide a mock; production uses the real client.
- **Testing**: Unit tests at the deepened seam with a mock implementation. Separately, maintain a small suite of **live contract tests** that run against the real external service on a schedule, to catch contract changes.

## Seam discipline

Before introducing a port, check that the seam is real:

- **One adapter means a hypothetical seam. Two adapters means a real one.** Don't introduce a port unless at least two adapters are justified — typically production + test. A single-adapter seam is just indirection without leverage.
- **Internal seams vs external seams.** A deep module can have internal seams (private to its implementation, used by its own tests) as well as the external seam at its interface. Don't expose internal seams through the public interface just because tests use them — that leaks implementation into the interface and kills depth.

For the in-process category, there is no seam (no adapter, no port). For local-substitutable, the seam is internal and usually doesn't reach the module's public interface. Only categories 3 and 4 generally warrant an exposed port.

## Testing principle -- replace, don't layer

Once tests exist at the deepened interface:

- Old unit tests on the formerly-shallow modules are waste -- **delete them**
- New tests assert observable outcomes through the public interface, not internal state
- Tests describe behavior, not implementation -- they survive internal refactors

The shallow-module tests were testing the wrong level. Deleting them isn't removing coverage; it's removing noise.

## Cross-reference

See references/deep-modules.md for the underlying philosophy, references/architecture-language.md for the shared vocabulary (seam, adapter, leverage, locality), references/mocking.md for where the seam sits, and references/interface-design.md for designing the port contract.
