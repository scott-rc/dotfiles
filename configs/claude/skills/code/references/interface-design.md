# Interface Design for Testability

Well-designed interfaces accommodate testing naturally -- no mocking tricks, no test-only hooks. Three principles.

## 1. Accept dependencies, don't create them

Functions and objects should receive their collaborators as parameters, not construct them internally. This lets tests substitute fakes, mocks, or alternate implementations without touching globals or environment.

```typescript
// TESTABLE: dependency is a parameter
function calculateDiscount(cart, discountRules) {
  return discountRules.apply(cart);
}

// HARDER to test: dependency is hidden behind a module-level constant
function calculateDiscount(cart) {
  return DEFAULT_DISCOUNT_RULES.apply(cart);
}
```

Dependency injection isn't a framework -- it's just passing things in.

## 2. Functional returns over mutations

Compute and return values. Don't mutate inputs in place.

```typescript
// EASIER to verify: pure function returns a value
function calculateDiscount(cart): Discount {
  return { amount: cart.subtotal * 0.1, reason: "LOYALTY_10" };
}

// HARDER to verify: mutation requires inspecting input state after the call
function applyDiscount(cart): void {
  cart.discount = cart.subtotal * 0.1;
  cart.discountReason = "LOYALTY_10";
}
```

The pure version has one assertion target (the returned `Discount`). The mutating version requires tests to inspect multiple fields on a mutated input -- more setup, more ways for a test to drift from intent.

Exception: sometimes mutation is the clearer design (e.g., append-only log). When in doubt, return.

## 3. Minimal interface complexity

Fewer methods and parameters mean fewer test scenarios required. Each extra parameter multiplies combinations.

```typescript
// Minimal: one clear call shape
function checkout(cart: Cart, payment: Payment): CheckoutResult;

// Bloated: flags and options compound the test surface
function checkout(
  cart: Cart,
  payment: Payment,
  options?: { dryRun?: boolean; skipEmail?: boolean; retryLimit?: number }
): CheckoutResult;
```

If you find yourself adding options flags to make the function "configurable", ask whether each flag represents a genuinely different behavior that callers need -- or whether you're adding it because it's convenient to have. Convenience flags bloat the test matrix without delivering user-facing value.

## Cross-reference

These principles align with references/deep-modules.md -- small interface, deep implementation. They also align with references/mocking.md -- if your internal modules are easy to substitute (DI), you won't feel the urge to mock collaborators you shouldn't mock.
