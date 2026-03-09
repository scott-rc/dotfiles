# Text Formatting

These rules apply to ALL text that leaves the session -- commit messages, PR titles, PR descriptions, PR comments, review comments, and any other text passed through `git` or `gh` CLI commands.

- **ASCII only**: Use `--` instead of em dashes, straight quotes instead of curly quotes, `...` instead of `…`. Non-ASCII gets corrupted through `gh` CLI.
- **Backticks for code references**, fenced code blocks for multi-line examples. This also neutralizes markdown metacharacters -- bare tildes (`~`) trigger strikethrough, bare asterisks (`*`) trigger emphasis. Wrap any operator or expression containing these in backticks (e.g., `level=~"info"`, `|~`, `err.*failed`).
- **Safe posting**: Use `~/.claude/skills/git/scripts/safe-text.sh` as an ASCII text filter. Pipe content or pass `--file <draft>`; it replaces common non-ASCII (em dashes, curly quotes, ellipsis, bullets, NBSP) and strips any remaining non-ASCII, then prints sanitized text to stdout. Capture into a variable or redirect to a file for `--body-file` / `-F body=@file`. Exits 1 if content is empty or subject/title exceeds length limit.

  ```bash
  # Commit message (capitalizes first letter, errors if >72 chars)
  MSG=$(echo "message" | ~/.claude/skills/git/scripts/safe-text.sh --commit-msg)
  # PR title (capitalizes first letter, errors if >70 chars)
  TITLE=$(echo "title" | ~/.claude/skills/git/scripts/safe-text.sh --title)
  # PR body (ASCII enforcement only -- redirect to file for --body-file)
  ~/.claude/skills/git/scripts/safe-text.sh --file "$DRAFT" > /tmp/pr-body.txt
  ```

- **No invented metrics**: MUST NOT cite specific numbers, percentages, multipliers, or performance claims unless they appear literally in the diff or commit message. Phrases like "reduces by 2.8x" or "cuts latency by 40%" are hallucination risks when the source material contains no such figures.
