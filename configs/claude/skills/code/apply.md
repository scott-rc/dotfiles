# Apply Coding Preferences

Load the user's coding style preferences, write code, and verify it matches.

## Instructions

1. **Load guidelines**: Follow [load-guidelines.md](load-guidelines.md). Note: Apply does not write tests, so skip testing-guidelines.md and test-environment.md.

2. **Write the code**: Apply the loaded preferences. When they conflict with existing project conventions (linter config, formatter, existing patterns), SHOULD follow project conventions.

3. **Self-check**: MUST verify code follows the loaded guidelines, paying special attention to naming, comments, nesting, error handling, and abstractions. For shell scripts, MUST run `shellcheck` and fix all warnings. If any violation is found, fix it and re-check. Repeat until all items pass.

4. **Run existing tests**: If the project has a test suite, run it (or the relevant subset) to confirm no regressions.

5. **Present results**: Present the code to the user with a summary of which preferences were applied, any conflicts resolved, and test suite status (pass/fail, number of tests run).
