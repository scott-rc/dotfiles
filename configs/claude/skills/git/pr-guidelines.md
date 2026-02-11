# PR Description Guidelines

## Title

- Match the first commit's title (the one that will be the squash commit)
- Keep under 72 characters (GitHub truncates longer titles in lists)
- Use imperative mood: "Add feature" not "Added feature"

## Format

- **Write prose, not bullets**: The default format is readable prose paragraphs. Only use bullet points when listing genuinely unrelated items where prose would be awkward.
- **No rigid templates**: Don't use fill-in-the-blank sections like `## Summary` followed by bullets and `## Test plan` followed by bullets. Write naturally.
- **MUST NOT wrap lines**: Do NOT wrap text to 72 characters or any other width. Write each thought as a single continuous line. GitHub renders markdown and handles line wrapping automatically.
- **Use backticks for code**: Wrap function names, variable names, file paths, etc.
- **Use code blocks for multi-line code**: Use triple backticks with language identifier for code examples.
- **ASCII only**: MUST use only ASCII characters in PR descriptions. Use `--` instead of em dashes, straight quotes instead of curly quotes, and `...` instead of `…`. Non-ASCII characters get corrupted when passed through `gh` CLI commands.

## Content

**The diff is the source of truth.** You MUST base the PR description on `git diff origin/<base>..HEAD`, NOT on commit history. The PR description MUST represent the net change between the current branch and the base branch — what a reviewer will actually see when they open the PR.

- **ALWAYS start with the diff**: Run `git diff origin/<base>..HEAD` and `git diff --stat origin/<base>..HEAD` to understand what is actually changing. This is what the reviewer sees. This is what the description MUST describe.
- **Commit history is supplementary only**: You MAY read `git log origin/<base>..HEAD` for context on *why* changes were made, but NEVER let commit history drive the structure or content of the description. Commits reflect the journey; the diff reflects the destination.
- **Describe the net change, not the journey**: If a bug was introduced in commit 1 and fixed in commit 3, do NOT mention the bug — it never existed in the base branch. If code was added then refactored, describe only the final form.
- **Focus on the "why"**: Explain the motivation and reasoning, not just what changed.
- **Include testing context**: Describe how the changes were verified, but as part of the narrative, not as a separate checklist.
- **Link issues**: Use "Fixes #123" to auto-close issues on merge; use "Related to #456" for referenced-but-not-fixed issues.

## Example

Title: `Add workspace-level snippet sharing`

Body:
```
Users in the same workspace frequently recreate identical snippets. This introduces a shared snippet library scoped to the workspace, with copy-on-edit semantics so personal modifications don't affect the original.

Storage uses the existing `snippets` table with an added `workspace_id` column and a composite index on `(workspace_id, name)`. The `SnippetService.list()` method now accepts an optional `workspaceId` parameter to fetch shared snippets alongside personal ones. Verified with integration tests against a multi-user workspace and confirmed no N+1 queries via `EXPLAIN ANALYZE`. Fixes #482.
```

## Updating PRs

- When the code changes significantly, update the description to match
- Preserve content appended by bots (BugBot, Dependabot, etc.)—it appears after your description
- If the PR scope changed, update the title too
