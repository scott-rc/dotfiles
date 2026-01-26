Summarize the useful information from our conversation and write it to a file for sharing on Slack.

## Slack Formatting

Slack has limited markdown support. Only use:

```
*bold*          _italic_        ~strikethrough~
`inline code`   [text](url)     > block quote
```

Code blocks work normally with triple backticks.

*Do not use:* `#` headers, tables, `---` rules, or nested formatting.

*Alternatives:*
- Headers → bold text on its own line
- Tables → code block with aligned columns:
  ```
  Name        Status    Count
  ─────────────────────────────
  Widget A    Active      123
  Widget B    Pending      45
  ```

## Instructions

1. Check if this command was invoked earlier in the conversation. If so, only summarize information since that previous invocation—don't repeat what was already shared.

2. Identify what's worth sharing: investigation findings, solutions to tricky problems, key decisions with context, or interesting discoveries.

3. Write to `tmp/slack_messages/<descriptive-name>.md` using kebab-case filenames.

4. Structure for skimmability:
   - Lead with a summary so readers know if they care to follow
   - Highlight key numbers and actionable items
   - Use code blocks for queries, snippets, or structured data

5. Output the full file path when done.

