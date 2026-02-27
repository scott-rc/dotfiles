# TypeScript Guidelines

## Type Annotations

- Always annotate return types on functions
- Let variable types be inferred from assignment
- Use `interface` for extensible object shapes
- Use `type` for unions, intersections, and aliases

## Functions

`function` declarations for top-level and exported functions. Arrow functions for callbacks and inline usage:

```ts
// top-level: function declaration
export function formatUser(user: User): string {
  return names
    .filter((n) => n.length > 0)
    .map((n) => n.trim())
    .join(" ");
}

// callback: arrow
const active = users.filter((u) => u.isActive);
```

## Imports

No preference — follow the project's formatter and import ordering conventions.
