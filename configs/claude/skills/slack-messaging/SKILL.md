---
name: slack-messaging
description: Enforces Slack formatting rules and tool selection when sending Slack messages — use when calling any Slack MCP `send_message` or `send_message_draft` tool.
user-invocable: false
---

# Slack Messaging

Applies correct tool selection and Slack-native formatting whenever sending a message via the Slack MCP integration.

## Operations

### Send Message

Enforce tool selection and format the message body using Slack-native markup before sending.

1. **Select the correct tool**: MUST use the Slack MCP `send_message` tool. MUST NOT use `send_message_draft` — `send_message` attributes the message to Claude, while the draft variant silently puts words in the user's mouth.

2. **Apply Slack-native formatting**: Slack uses its own markup, not standard Markdown. Apply the following rules to the message body:

   Supported formatting:
   - `*bold*` — bold text
   - `_italic_` — italic text
   - `~strikethrough~` — strikethrough text
   - `` `inline code` `` — inline code
   - Triple backtick code blocks — work normally
   - `> block quote` — block quotes
   - `<url|text>` — links

   MUST NOT use:
   - `#` headers — use `*bold text*` on its own line instead
   - Markdown tables — use code blocks with aligned columns instead (see step 4)
   - `---` horizontal rules
   - Nested formatting

3. **Sanitize dangerous characters**: These characters trigger unintended formatting in Slack and MUST be handled:
   - **Tilde `~`**: Slack renders `~text~` as strikethrough. When `~` means "approximately", MUST replace with "approx", wrap in a code span (`` `~280K` ``), or spell it out. MUST NOT leave a bare `~` adjacent to text.
   - **Asterisk `*`**: Slack renders `*text*` as bold. Only use `*` when intentionally bolding. When `*` appears in formulas or math expressions (bad: `28K × 72s = *~560` → good: `` `28K × 72s = ~560` ``), wrap the expression in a code span or code block.
   - **Underscore `_`**: Slack renders `_text_` as italic. When `_` appears in identifiers like `variable_name`, wrap in a code span.

4. **Format tabular data as code blocks**: When presenting tabular data, use a fenced code block with aligned columns:

   ```
   Metric          Before       After
   ────────────────────────────────────
   Avg duration    16ms         23,857ms
   p95 duration    —            185,798ms
   ```
