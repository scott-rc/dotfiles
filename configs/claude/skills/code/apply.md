# Apply Coding Preferences

Load the user's coding style preferences, write code, and verify it matches.

## Instructions

1. **Read general preferences**: MUST load [general-guidelines.md](general-guidelines.md). These apply to all languages.

2. **Identify target language**: Determine from the user's request, file extension, or project context.

3. **Read language-specific preferences** (if available):
   - **TypeScript**: [typescript-guidelines.md](typescript-guidelines.md)

   If no file exists for the target language, use only the general guidelines.

4. **Write the code**: Apply the loaded preferences. When they conflict with existing project conventions (linter config, formatter, existing patterns), SHOULD follow project conventions.

5. **Self-check**: MUST verify code follows the loaded guidelines, paying special attention to naming, comments, nesting, error handling, and abstractions. If any violation is found, fix it and re-check. Repeat until all items pass.

6. **Present results**: Present the code to the user with a summary of which preferences were applied and any conflicts resolved.
