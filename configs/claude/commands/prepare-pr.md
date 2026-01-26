# Prepare PR

Prepare this branch for a PR by committing any outstanding changes and squashing any existing commits into a single one with a PR-ready commit message.

## Instructions

1. Commit any outstanding changes using `git commit -m "commit message"`. If there are no outstanding changes, skip this step.
2. Detect the base branch by running `git rev-parse --abbrev-ref origin/HEAD 2>/dev/null | sed 's|origin/||' || echo 'main'`.
3. Get the list of commits to squash using `git show <base>..HEAD --reverse`.
4. Analyze the changes to understand what work was done and why.
5. Squash all commits into one:

   ```bash
   git reset --soft <base>
   git commit
   ```

6. Format the commit message following this structure:

   ```
   <concise PR title - one line summarizing the change>

   <explain the motivation and reasoning behind these changes>
   ```

## Commit Message Guidelines

- **Title**: Should be a clear, concise description suitable as a PR title (ideally under 50 characters, max 72).
- **Context**: Explain _why_ these changes were made, not just _what_ changed. If you are unsure, ask the user. These lines can be longer than 72 characters as they will be wrapped by GitHub.
- **Code formatting**: Wrap all code-related text with backticks, including function names, variable names, type names, file paths, package names, flag names, and environment variables.
