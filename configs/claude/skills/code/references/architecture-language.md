# Architecture Language

Shared vocabulary for the Architect operation and the Review checklist's structural section. Use these terms exactly — don't drift into "component", "service", "API", or "boundary" when talking about architectural structure. Consistent language keeps design conversations focused on structure instead of spelling.

("Boundary" in the broader sense — system boundary, network boundary, boundary values in tests — stays in the vocabulary. Only the architectural sense, as a synonym for where an interface lives, is replaced by **seam**.)

## Terms

**Module**
Anything with an interface and an implementation. Deliberately scale-agnostic — applies equally to a function, class, package, or tier-spanning slice.
_Avoid_: unit, component, service.

**Interface**
Everything a caller must know to use the module correctly. Includes the type signature, but also invariants, ordering constraints, error modes, required configuration, and performance characteristics.
_Avoid_: API, signature — both are too narrow; they refer only to the type-level surface.

**Implementation**
What's inside a module — its body of code. Distinct from **Adapter**: a module can be a small adapter with a large implementation (a Postgres repo) or a large adapter with a small implementation (an in-memory fake). Reach for "adapter" when the seam is the topic; "implementation" otherwise.

**Depth**
Leverage at the interface — how much behavior a caller (or test) can exercise per unit of interface they have to learn. A module is **deep** when a large amount of behavior sits behind a small interface. A module is **shallow** when the interface is nearly as complex as the implementation.

**Seam** _(Michael Feathers)_
A place where you can alter behavior without editing in that place. The *location* at which a module's interface lives. Choosing where to put the seam is its own design decision, distinct from what goes behind it.
_Avoid_: "boundary" as a synonym — overloaded with DDD's bounded context. Use "seam" or "interface".

**Adapter**
A concrete thing that satisfies an interface at a seam. Describes *role* (what slot it fills), not substance (what's inside).

**Leverage**
What callers get from depth — more capability per unit of interface they have to learn. One implementation pays back across N call sites and M tests.

**Locality**
What maintainers get from depth — change, bugs, knowledge, and verification concentrate at one place rather than spreading across callers. Fix once, fixed everywhere.

## Principles

- **Depth is a property of the interface, not the implementation.** A deep module can be internally composed of small, swappable parts — they just aren't part of the interface. A module can have **internal seams** (private to its implementation, used by its own tests) as well as the **external seam** at its interface. Don't expose internal seams through the public interface just because tests use them.
- **The deletion test.** Imagine deleting the module. If complexity vanishes, the module was a pass-through — it wasn't hiding anything. If complexity reappears and spreads across N callers, the module was earning its keep. Use this as the primary filter when scanning for shallow modules in discovery.
- **The interface is the test surface.** Callers and tests cross the same seam. If you want to test *past* the interface, the module is probably the wrong shape — reshape the module rather than scaffolding test-only entry points.
- **One adapter means a hypothetical seam. Two adapters means a real one.** Don't introduce a port unless at least two adapters are justified (typically production + test). A single-adapter seam is just indirection — pay for the seam only when something actually varies across it.

## Relationships

- A **Module** has exactly one **Interface** (the surface it presents to callers and tests).
- **Depth** is a property of a **Module**, measured against its **Interface**.
- A **Seam** is where a **Module**'s **Interface** lives.
- An **Adapter** sits at a **Seam** and satisfies the **Interface**.
- **Depth** produces **Leverage** for callers and **Locality** for maintainers.

## Rejected framings

- **Depth as the ratio of implementation-lines to interface-lines** (literal reading of Ousterhout): rewards padding the implementation. Use depth-as-leverage instead — a small implementation behind a small interface can still be deep if it concentrates behavior callers would otherwise reinvent.
- **"Interface" as just the TypeScript `interface` keyword or a class's public methods**: too narrow. Interface here includes every fact a caller must know — invariants, ordering, error modes, required config, performance characteristics.
- **"Boundary" as a synonym for seam**: overloaded with DDD's bounded context. Say **seam** or **interface**.

## Cross-reference

- [deep-modules.md](deep-modules.md) — Ousterhout's small-interface/large-implementation framing, applied.
- [dependency-categories.md](dependency-categories.md) — How the seam choice (in-process, local-substitutable, ports-and-adapters, mock-external) determines testing strategy.
- [interface-design.md](interface-design.md) — Testability-driven interface design at the seam.
- [refactor-smells.md](refactor-smells.md) — Structural smells that signal a deepening opportunity.
