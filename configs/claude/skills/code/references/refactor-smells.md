# Refactor Candidates

Smells to watch for after a TDD green cycle, when reviewing code, or when scoping a `code architect` invocation. Each points to a specific kind of restructuring.

## Local smells (address inline during TDD refactor step)

- **Duplication** -- 3+ repetitions of the same logic → extract function/class. Two is a coincidence; three is a pattern.
- **Long methods** -- step count or cyclomatic complexity makes the method hard to read → break into private helpers. Keep tests on the public interface; the helpers shouldn't get their own test file.
- **Feature envy** -- a method spends most of its time poking at another object's data → move the logic to where the data lives.
- **Primitive obsession** -- a bag of strings/numbers/booleans that always travel together → introduce a value object that names the concept.

## Structural smells (often warrant `code architect`)

- **Shallow modules** (see references/deep-modules.md) -- large interface hiding trivial implementation, or a thin wrapper that adds no abstraction. Single small shallowness can be fixed inline; broad shallowness across multiple modules deserves a brief via `code architect`.
- **Tightly-coupled cluster** -- three or more modules that always change together, share internal types, or can't be tested independently. Consider combining into one deep module or introducing a shared interface (port).
- **Pure-logic-extracted-only-for-testing** -- a function lives in isolation solely so tests can reach it. The test surface should be the real public API; if the only way to test is through a scaffold, the real API is wrong.

## Derived smells from new code

- **"This would be easier if ..."** -- new code reveals friction in existing code. Note it, finish the current cycle, then decide: fix inline (small) or write a brief via `code architect` (structural).

## Cross-reference

See references/deep-modules.md for the "small interface, deep implementation" principle, references/interface-design.md for testability-driven design, and references/dependency-categories.md for how dependency types constrain the refactor strategy.
