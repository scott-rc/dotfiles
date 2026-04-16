# Deep Modules

Per John Ousterhout's _A Philosophy of Software Design_, a **deep module** has a **small interface + lots of implementation**. The interface reveals just enough to be used; the implementation does the heavy lifting behind it.

The opposite -- a **shallow module** with a **large interface + little implementation** -- fails to provide meaningful abstraction. The caller has to understand almost as much to use the module as they would without it.

## Why depth matters

- **Cognitive load**: callers reason about the interface, not the internals
- **Testability**: tests exercise the public surface, not scattered helpers
- **Refactor resilience**: internal restructuring doesn't cascade into caller code or test rewrites
- **AI navigation**: agents reading the codebase can understand the module from its interface alone

## Three questions when designing an interface

1. **Can the number of exposed methods be reduced?** Combine related operations; hide internal helpers.
2. **Can parameter signatures be simplified?** Replace flag bags with specific operations; use value objects instead of primitive clusters.
3. **Can more internal complexity be concealed from callers?** If callers have to sequence two calls in a specific order, that sequence probably belongs inside the module.

## Signals that a module is too shallow

- Exports a helper that's only consumed by one caller
- Exposes internal data structures that could be hidden
- Has methods whose names describe implementation steps rather than capabilities
- Callers routinely chain 3+ methods in a specific order to accomplish one conceptual operation
- Unit tests on the module are mostly setup glue with trivial assertions

See references/refactor-smells.md for a broader catalog, and references/dependency-categories.md for how dependency type affects the depth strategy.

## When to act on shallowness

Small, localized shallowness can be addressed inline during a TDD refactor step. Broad or cross-cutting shallowness (multiple modules clustering, network boundaries, ports-and-adapters opportunities) is a signal to invoke `code architect <target>` -- which produces a refactor RFC with parallel design alternatives rather than trying to fix the depth problem inline.
