# General Coding Guidelines

## Naming

Short and contextual. Narrow-scope variables get short names (`i`, `el`, `err`). Exported functions and types get descriptive names that read well at the call site.

## Comments

- Document public APIs per language convention
- Brief inline "why" comments for non-obvious logic
- No comments restating what the code does

## Error Handling

Defensive at system boundaries — API handlers, external service calls, user input parsing. Internally, trust your own code and let unexpected errors propagate.

## Control Flow

Guard clauses over nested conditionals. Return or throw early to keep the happy path unindented:

```ts
function process(input: Input): Result {
  if (!input.isValid) {
    throw new ValidationError("invalid input");
  }

  if (input.isEmpty) {
    return emptyResult();
  }

  // happy path at top level
  return transform(input);
}
```

## Abstractions

Inline-first. Repeat code up to 3 times before extracting. When you do extract, the abstraction MUST have a clear interface and a name that describes what it does, not how it's used.

## Testing

- Test behavior, not implementation
- Minimal mocking — prefer integration-style tests with real dependencies where practical
- Descriptive test names that state the scenario and expected outcome

## Strings

Plain quotes for static strings. Template literals only when interpolating.
