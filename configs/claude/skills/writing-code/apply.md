# Apply Coding Preferences

Load the user's coding style preferences, write code, and verify it matches.

## Instructions

1. **Read general preferences**: Load [general-guidelines.md](general-guidelines.md). These apply to all languages.

2. **Identify target language**: Determine from the user's request, file extension, or project context.

3. **Read language-specific preferences** (if available):

   | Language   | File                                               |
   |------------|----------------------------------------------------|
   | TypeScript | [typescript-guidelines.md](typescript-guidelines.md) |

   If no file exists for the target language, use only the general guidelines.

4. **Write the code**: Apply the loaded preferences. When they conflict with existing project conventions (linter config, formatter, existing patterns), follow project conventions.

5. **Self-check**: Before presenting code, verify:
   - Naming follows scope-appropriate length conventions
   - Comments explain "why", not "what"
   - Guard clauses reduce nesting where applicable
   - Error handling is defensive at boundaries, not internally
   - No premature abstractions — inline until 3+ repetitions
