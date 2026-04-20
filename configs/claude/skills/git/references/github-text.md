# Text Formatting

These rules apply to ALL text that leaves the session -- commit messages, PR titles, PR descriptions, PR comments, review comments, and any other text passed through `git` or `gh` CLI commands.

- **ASCII only**: Use `--` instead of em dashes, straight quotes instead of curly quotes, `...` instead of `…`. Non-ASCII gets corrupted through `gh` CLI.
- **Backticks for code references**, fenced code blocks for multi-line examples. This also neutralizes markdown metacharacters -- bare tildes (`~`) trigger strikethrough, bare asterisks (`*`) trigger emphasis. Wrap any operator or expression containing these in backticks (e.g., `level=~"info"`, `|~`, `err.*failed`).
- **Safe posting**: Write text to a PR-scoped temp file using Bash, then sanitize in place with `~/.claude/skills/git/scripts/sanitize.sh`. The script replaces common non-ASCII (em dashes, curly quotes, ellipsis, bullets, NBSP) and strips any remaining non-ASCII. On success the file is modified in place (exit 0). Exits 1 if content is empty or subject/title exceeds length limit (file unchanged).

  **Temp path convention** (scoped to keep parallel agents collision-free):
  - Commit message: `./tmp/commit-msg.txt` (session-scoped; only one active commit draft at a time)
  - PR title / body (update mode): `./tmp/pr/<pr_number>/{title,body}.txt`
  - PR title / body (create mode): `./tmp/pr/${BRANCH_SLUG}/{title,body}.txt` where `BRANCH_SLUG` is the branch name with `/` replaced by `-`
  - Review reply: `./tmp/pr/<pr_number>/reply-<comment_id>.txt`
  - PR-level comment: `./tmp/pr/<pr_number>/comment.txt`

  Create the directory once per PR with `mkdir -p ./tmp/pr/<pr_number>` (or `${BRANCH_SLUG}`), then heredoc the content into the target file.

  ```bash
  # Commit message (capitalizes first letter, errors if >72 chars)
  mkdir -p ./tmp && cat <<'EOF' > ./tmp/commit-msg.txt
  ...
  EOF
  ~/.claude/skills/git/scripts/sanitize.sh --commit-msg ./tmp/commit-msg.txt
  git-spice commit create -m "$(cat ./tmp/commit-msg.txt)" --no-prompt

  # PR title (capitalizes first letter, errors if >70 chars) and body, update mode
  mkdir -p ./tmp/pr/<pr_number>
  # Heredoc title to ./tmp/pr/<pr_number>/title.txt and body to ./tmp/pr/<pr_number>/body.txt, then:
  ~/.claude/skills/git/scripts/sanitize.sh --title ./tmp/pr/<pr_number>/title.txt
  ~/.claude/skills/git/scripts/sanitize.sh        ./tmp/pr/<pr_number>/body.txt

  # Review reply
  mkdir -p ./tmp/pr/<pr_number>
  # Heredoc reply to ./tmp/pr/<pr_number>/reply-<comment_id>.txt, then:
  ~/.claude/skills/git/scripts/sanitize.sh ./tmp/pr/<pr_number>/reply-<comment_id>.txt
  gh api repos/{owner}/{repo}/pulls/{pull_number}/comments/{comment_id}/replies \
    -F body=@./tmp/pr/<pr_number>/reply-<comment_id>.txt

  # PR-level comment
  mkdir -p ./tmp/pr/<pr_number>
  # Heredoc comment to ./tmp/pr/<pr_number>/comment.txt, then:
  ~/.claude/skills/git/scripts/sanitize.sh ./tmp/pr/<pr_number>/comment.txt
  gh pr comment {pr_number} --repo {owner}/{repo} --body-file ./tmp/pr/<pr_number>/comment.txt
  ```

- **No invented metrics**: MUST NOT cite specific numbers, percentages, multipliers, or performance claims unless they appear literally in the diff or commit message. Phrases like "reduces by 2.8x" or "cuts latency by 40%" are hallucination risks when the source material contains no such figures.

## Concurrent Agents

The `./tmp/pr/<pr_number>/` or `./tmp/pr/${BRANCH_SLUG}/` scoping above keeps parallel agents from clobbering each other. MUST NOT fall back to shared paths like `./tmp/pr-body.txt` — parallel agents each working on their own PR will overwrite a sibling agent's file.
