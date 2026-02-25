# Apply Coding Preferences

Load the user's coding style preferences, write code, and verify it matches.

## Instructions

1. **Read general preferences**: MUST load [general-guidelines.md](general-guidelines.md). These apply to all languages.

2. **Identify target language**: Determine from the user's request, file extension, or project context.

3. **Read language-specific preferences** (if available):
   - **TypeScript**: [typescript-guidelines.md](typescript-guidelines.md)
   - **Go**: [go-guidelines.md](go-guidelines.md)
   - **Rust**: [rust-guidelines.md](rust-guidelines.md)
   - **Bash / Fish**: [shell-guidelines.md](shell-guidelines.md)

   If no file exists for the target language, use only the general guidelines.

4. **Write the code**: Apply the loaded preferences. When they conflict with existing project conventions (linter config, formatter, existing patterns), SHOULD follow project conventions.

5. **Self-check**: MUST verify code follows the loaded guidelines, paying special attention to naming, comments, nesting, error handling, and abstractions. For shell scripts, MUST run `shellcheck` and fix all warnings. If any violation is found, fix it and re-check. Repeat until all items pass.

6. **Run existing tests**: If the project has a test suite, run it (or the relevant subset) to confirm no regressions.

7. **Present results**: Present the code to the user with a summary of which preferences were applied, any conflicts resolved, and test suite status (pass/fail, number of tests run).
