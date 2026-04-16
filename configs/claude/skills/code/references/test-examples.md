# Good vs Bad Tests

Concrete examples showing which tests survive refactoring and which don't.

## Good tests

**Integration-style**: test through the same public interface real callers use. Describe a _capability_ the system provides.

Characteristics:
- Tests behavior users/callers care about
- Uses public API only
- Survives internal refactors (renaming a private function, restructuring internals)
- Describes WHAT the system does, not HOW
- One logical assertion per test

Example (TypeScript):

```typescript
// GOOD: tests observable behavior through the public interface
test("user can checkout with valid cart", async () => {
  const cart = createCart();
  cart.add(product);
  const result = await checkout(cart, paymentMethod);
  expect(result.status).toBe("confirmed");
});
```

A good test name reads like a specification: "user can checkout with valid cart" tells you a capability exists. Rename `paymentService.process` to `paymentService.charge` internally and this test still passes.

## Bad tests

**Implementation-detail tests**: coupled to the internals, not the behavior.

Red flags:
- Mocking internal collaborators (things you control)
- Testing private methods directly
- Asserting on call counts or call order
- Test breaks when refactoring without behavior change
- Test name describes HOW, not WHAT
- Verifying through external means (e.g., direct DB query) instead of the public interface

Example (TypeScript):

```typescript
// BAD: tests implementation, not behavior
test("checkout calls paymentService.process", async () => {
  const mockPayment = jest.mock(paymentService);
  await checkout(cart, payment);
  expect(mockPayment.process).toHaveBeenCalledWith(cart.total);
});
```

Rename `process` → `charge` and this test breaks despite behavior being correct. The test name describes HOW (what method was called) instead of WHAT (that payment was successfully processed).

Another common anti-pattern — bypassing the interface to verify:

```typescript
// BAD: reaches around the public API to verify
test("createUser saves to database", async () => {
  await createUser({ name: "Alice" });
  const row = await db.query("SELECT * FROM users WHERE name = ?", ["Alice"]);
  expect(row).toBeDefined();
});

// GOOD: verifies through the interface real callers use
test("createUser makes user retrievable", async () => {
  const user = await createUser({ name: "Alice" });
  const retrieved = await getUser(user.id);
  expect(retrieved.name).toBe("Alice");
});
```

The good version still passes if the underlying storage switches from SQL to a document store. The bad version breaks even though behavior is unchanged.

## Applying this to other languages

The principle is language-agnostic:
- **Go**: don't assert on unexported-package state via reflection; don't mock interfaces that belong to your own package. Use table-driven tests against the public API.
- **Rust**: don't use `#[cfg(test)]` hooks to peek into private state; test through the public module surface. If you need a test-only constructor, keep it minimal and justified.
- **Shell**: test script output and exit code, not intermediate variable values.

See also: references/mocking.md for when mocking is appropriate; references/interface-design.md for designing interfaces that resist bad tests.
