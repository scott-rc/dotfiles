# When to Mock

Mock at **system boundaries only** -- never inside the system.

## Mock at boundaries

- External APIs (payment, email, SaaS services)
- Databases (prefer a test DB or in-memory substitute like PGLite when available)
- Time and randomness
- File system (sometimes -- an in-memory filesystem is often better)

## Do NOT mock

- Your own classes and modules
- Internal collaborators
- Anything you control

Mocking internal collaborators couples tests to the current decomposition. A refactor that rearranges internals -- without changing behavior -- will break those tests. That's the warning sign: if renaming an internal function breaks your tests, your tests are testing implementation, not behavior.

## Designing boundaries for mockability

### 1. Dependency injection -- accept dependencies, don't create them

```typescript
// EASY to mock -- client is injected
function processPayment(order, paymentClient) {
  return paymentClient.charge(order.total);
}

// HARD to mock -- client is constructed inside
function processPayment(order) {
  const client = new StripeClient(process.env.STRIPE_KEY);
  return client.charge(order.total);
}
```

The injected version accepts a mock, a fake, or the real thing -- all transparently. The constructor-inside version requires environment manipulation or module-level mocking tricks.

### 2. Prefer SDK-style interfaces over generic fetchers

A specific function per operation, not one generic function with conditional logic:

```typescript
// GOOD: each function is independently mockable
const api = {
  getUser: (id) => fetch(`/users/${id}`),
  getOrders: (userId) => fetch(`/users/${userId}/orders`),
  createOrder: (data) => fetch('/orders', { method: 'POST', body: data }),
};

// BAD: mocking this requires conditional logic inside the mock
const api = {
  fetch: (endpoint, options) => fetch(endpoint, options),
};
```

The SDK approach means:
- Each mock returns one specific shape -- no branching in test setup
- Tests show clearly which endpoints are exercised
- Type safety per endpoint

## Cross-reference

See references/interface-design.md for a broader framing of testability-driven interface design. See references/dependency-categories.md for a taxonomy that ties dependency type (in-process, local-substitutable, ports-and-adapters, mock-boundary) to testing strategy.
