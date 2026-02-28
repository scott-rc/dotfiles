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

Prefer guard clauses and early returns over nested conditionals.

## Abstractions

Inline-first. Extract only at 3 or more repetitions. Extracted abstractions MUST have a clear interface and a name that describes what they do, not how they're used.

## Strings

Plain quotes for static strings. Template literals only when interpolating.
