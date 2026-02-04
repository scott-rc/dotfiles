# PR Description Guidelines

When creating or updating a PR description:

- **NEVER wrap lines**: Do NOT wrap text to 72 characters or any other width. Write each thought as a single continuous line. GitHub renders markdown and handles line wrapping automatically.
- **Summarize all commits**: Write a single cohesive summary covering all commits on the branch, not just the first one. Review the full commit history with `git log main..HEAD` (or appropriate base branch).
- **Focus on the "why"**: Explain the motivation and reasoning, not just what changed.
- **Use backticks for code**: Wrap function names, variable names, file paths, etc.
