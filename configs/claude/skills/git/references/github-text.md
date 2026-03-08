# GitHub Text

These rules apply to ALL text written to GitHub -- PR titles, PR descriptions, PR comments, review comments, and any other text passed through `gh` CLI commands.

- **ASCII only**: Use `--` instead of em dashes, straight quotes instead of curly quotes, `...` instead of `…`. Non-ASCII gets corrupted through `gh` CLI.
- **Backticks for code references**, fenced code blocks for multi-line examples. This also neutralizes markdown metacharacters -- bare tildes (`~`) trigger strikethrough, bare asterisks (`*`) trigger emphasis. Wrap any operator or expression containing these in backticks (e.g., `level=~"info"`, `|~`, `err.*failed`).
- **Safe posting**: Use `~/.claude/skills/git/scripts/safe-text.sh` to create safe temp files. Pipe content or pass `--file <draft>`; it replaces common non-ASCII (em dashes, curly quotes, ellipsis, bullets, NBSP) and strips any remaining non-ASCII, then returns the temp file path. Use that path with `--body-file` (gh pr) or `-F body=@file` (gh api). Returns exit 1 if content is empty.

  ```bash
  # Commit message (capitalizes first letter, warns if >72 chars)
  echo "message" | ~/.claude/skills/git/scripts/safe-text.sh --commit-msg --prefix commit-msg
  # PR title (capitalizes first letter, warns if >70 chars)
  echo "title" | ~/.claude/skills/git/scripts/safe-text.sh --title --prefix pr-title
  # PR body (ASCII enforcement only)
  ~/.claude/skills/git/scripts/safe-text.sh --file "$DRAFT" --prefix pr-body
  ```

- **No invented metrics**: MUST NOT cite specific numbers, percentages, multipliers, or performance claims unless they appear literally in the diff or commit message. Phrases like "reduces by 2.8x" or "cuts latency by 40%" are hallucination risks when the source material contains no such figures.
