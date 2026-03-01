# GitHub Text

These rules apply to ALL text written to GitHub -- PR titles, PR descriptions, PR comments, review comments, and any other text passed through `gh` CLI commands.

- **ASCII only**: Use `--` instead of em dashes, straight quotes instead of curly quotes, `...` instead of `…`. Non-ASCII gets corrupted through `gh` CLI.
- **Backticks for code references**, fenced code blocks for multi-line examples. This also neutralizes markdown metacharacters -- bare tildes (`~`) trigger strikethrough, bare asterisks (`*`) trigger emphasis. Wrap any operator or expression containing these in backticks (e.g., `level=~"info"`, `|~`, `err.*failed`).
- **Safe posting**: Write multi-line bodies to a temp file and use `-F body=@file` instead of inline strings or heredocs.
- **No invented metrics**: Never cite specific numbers, percentages, multipliers, or performance claims unless they appear literally in the diff or commit message. Phrases like "reduces by 2.8x" or "cuts latency by 40%" are hallucination risks when the source material contains no such figures.
