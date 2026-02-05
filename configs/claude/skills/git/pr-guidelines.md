# PR Description Guidelines

When creating or updating a PR description:

## Title

- Match the first commit's title (the one that will be the squash commit)
- Keep under 72 characters (GitHub truncates longer titles in lists)
- Use imperative mood: "Add feature" not "Added feature"

## Format

- **Write prose, not bullets**: The default format is readable prose paragraphs. Only use bullet points when listing genuinely unrelated items where prose would be awkward.
- **No rigid templates**: Don't use fill-in-the-blank sections like `## Summary` followed by bullets and `## Test plan` followed by bullets. Write naturally.
- **NEVER wrap lines**: Do NOT wrap text to 72 characters or any other width. Write each thought as a single continuous line. GitHub renders markdown and handles line wrapping automatically.
- **Use backticks for code**: Wrap function names, variable names, file paths, etc.
- **Use code blocks for multi-line code**: Use triple backticks with language identifier for code examples.

## Content

- **Synthesize commits into a narrative**: Read all commits with `git log main..HEAD` and weave them into a coherent story. Don't just list what each commit did—explain the overall change and why it matters.
- **Describe the net change, not the journey**: The PR description should reflect what's different between the base branch and the final state—not intermediate bugs, refactors, or course-corrections that happened along the way. If a bug was introduced in commit 1 and fixed in commit 3, don't mention the bug at all—it never existed in the base branch. Use `git diff main..HEAD` as your source of truth for what actually changed.
- **Focus on the "why"**: Explain the motivation and reasoning, not just what changed.
- **Include testing context**: Describe how the changes were verified, but as part of the narrative, not as a separate checklist.
- **Link issues**: Use "Fixes #123" to auto-close issues on merge; use "Related to #456" for referenced-but-not-fixed issues.
- **Single vs multiple commits**: For one commit, expand on its message. For multiple commits, read `git log main..HEAD` and synthesize into a narrative—don't list each commit separately.

## Updating PRs

- When the code changes significantly, update the description to match
- Preserve content appended by bots (BugBot, Dependabot, etc.)—it appears after your description
- If the PR scope changed, update the title too
