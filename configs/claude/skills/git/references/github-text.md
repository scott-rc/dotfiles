# Text Formatting

These rules apply to ALL text that leaves the session -- commit messages, PR titles, PR descriptions, PR comments, review comments, and any other text passed through `git` or `gh` CLI commands.

- **ASCII only**: Use `--` instead of em dashes, straight quotes instead of curly quotes, `...` instead of `…`. Non-ASCII gets corrupted through `gh` CLI.
- **Backticks for code references**, fenced code blocks for multi-line examples. This also neutralizes markdown metacharacters -- bare tildes (`~`) trigger strikethrough, bare asterisks (`*`) trigger emphasis. Wrap any operator or expression containing these in backticks (e.g., `level=~"info"`, `|~`, `err.*failed`).
- **Safe posting**: Write text to a temp file using Bash (`mkdir -p ./tmp && cat <<'EOF' > ./tmp/<file>.txt` ... `EOF`), then sanitize in place with `~/.claude/skills/git/scripts/sanitize.sh`. The script replaces common non-ASCII (em dashes, curly quotes, ellipsis, bullets, NBSP) and strips any remaining non-ASCII. On success the file is modified in place (exit 0). Exits 1 if content is empty or subject/title exceeds length limit (file unchanged).

  ```bash
  # Commit message (capitalizes first letter, errors if >72 chars)
  # Write message to ./tmp/commit-msg.txt using Bash heredoc, then:
  ~/.claude/skills/git/scripts/sanitize.sh --commit-msg ./tmp/commit-msg.txt
  gs commit create -m "$(cat ./tmp/commit-msg.txt)" --no-prompt

  # PR title (capitalizes first letter, errors if >70 chars)
  # Write title to ./tmp/pr-title.txt using Bash heredoc, then:
  ~/.claude/skills/git/scripts/sanitize.sh --title ./tmp/pr-title.txt

  # PR body (ASCII enforcement only)
  # Write body to ./tmp/pr-body.txt using Bash heredoc, then:
  ~/.claude/skills/git/scripts/sanitize.sh ./tmp/pr-body.txt

  # Review reply
  # Write reply to ./tmp/reply.txt using Bash heredoc, then:
  ~/.claude/skills/git/scripts/sanitize.sh ./tmp/reply.txt
  gh api repos/{owner}/{repo}/pulls/{pull_number}/comments/{comment_id}/replies -F body=@./tmp/reply.txt

  # PR comment
  # Write comment to ./tmp/comment.txt using Bash heredoc, then:
  ~/.claude/skills/git/scripts/sanitize.sh ./tmp/comment.txt
  gh pr comment {pr_number} --repo {owner}/{repo} --body-file ./tmp/comment.txt
  ```

- **No invented metrics**: MUST NOT cite specific numbers, percentages, multipliers, or performance claims unless they appear literally in the diff or commit message. Phrases like "reduces by 2.8x" or "cuts latency by 40%" are hallucination risks when the source material contains no such figures.

## Concurrent Agents

When multiple agents run in parallel and each uses the safe posting pattern, temp file paths MUST be unique per agent. Use a distinguishing suffix -- PR number for update mode, sanitized branch name for create mode -- to prevent clobbering.

```
./tmp/pr-20135-body.txt    # good: PR-specific
./tmp/pr-body.txt          # bad: shared path, clobbered by parallel agents
```
