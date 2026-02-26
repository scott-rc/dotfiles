# GitHub Text

These rules apply to ALL text written to GitHub -- PR titles, PR descriptions, PR comments, review comments, and any other text passed through `gh` CLI commands.

- **ASCII only**: Use `--` instead of em dashes, straight quotes instead of curly quotes, `...` instead of `…`. Non-ASCII gets corrupted through `gh` CLI.
- **Backticks for code references**, fenced code blocks for multi-line examples.
- **Safe posting**: Write multi-line bodies to a temp file and use `-F body=@file` instead of inline strings or heredocs.
