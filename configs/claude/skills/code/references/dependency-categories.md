# Dependency Categories

When assessing a module for deepening (TDD refactor step or `code architect` design), classify each dependency. The category determines the deepening strategy and the testing approach.

## 1. In-process

Pure computation, in-memory state, no I/O. No network, no disk, no system calls.

- **Deepening strategy**: Always deepenable. Merge the shallow modules; test directly through the combined interface.
- **Testing**: Unit tests at the deepened interface boundary. No fakes or mocks required.

## 2. Local-substitutable

Has an external dependency, but a local test stand-in exists and runs in the test suite.

Examples: PGLite for Postgres, in-memory filesystem, an in-memory cache instead of Redis.

- **Deepening strategy**: Deepenable if the substitute is viable for the behaviors under test. The deepened module owns the logic; tests run against the stand-in; production uses the real dependency.
- **Testing**: Integration tests through the public interface, with the stand-in running. Tests exercise real code paths end-to-end.

## 3. Remote but owned (Ports & Adapters)

Your own services across a network boundary -- internal APIs, microservices, message queues you operate.

- **Deepening strategy**: Define a **port** (interface) at the module boundary. The deepened module owns the logic; the transport is injected via the port. Production gets an HTTP/gRPC/queue adapter; tests get an in-memory adapter.
- **Testing**: Boundary tests use the in-memory adapter. Tests exercise the module as one deep unit, even though the system is deployed across a network.
- **RFC phrasing**: "Define a shared interface (port), implement an HTTP adapter for production and an in-memory adapter for testing, so the logic can be tested as one deep module even though it's deployed across a network boundary."

## 4. True external (Mock-boundary)

Third-party services you don't control -- Stripe, Twilio, SaaS APIs.

- **Deepening strategy**: Mock at the boundary. The deepened module takes the external dependency as an injected port (same pattern as ports-and-adapters); tests provide a mock; production uses the real client.
- **Testing**: Unit tests at the deepened boundary with a mock implementation. Separately, maintain a small suite of **live contract tests** that run against the real external service on a schedule, to catch contract changes.

## Testing principle -- replace, don't layer

Once boundary tests exist at the deepened interface:

- Old unit tests on the formerly-shallow modules are waste -- **delete them**
- New tests assert observable outcomes through the public interface, not internal state
- Tests describe behavior, not implementation -- they survive internal refactors

The shallow-module tests were testing the wrong level. Deleting them isn't removing coverage; it's removing noise.

## Cross-reference

See references/deep-modules.md for the underlying philosophy, references/mocking.md for where the boundary sits, and references/interface-design.md for designing the port contract.
