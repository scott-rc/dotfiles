# Review Prompt

Evaluate a session task prompt against best practices, report findings, and offer to improve it.

## Instructions

1. **Locate the prompt**:
   - If the user pastes prompt text directly, use it as-is
   - If the user provides a file path, read the file
   - If the user says "clipboard" or "from clipboard", read from `pbpaste`
   - If unclear, ask the user to paste the prompt, provide a file path, or confirm clipboard

2. **Analyze structure**:
   Check for these sections (per [create-prompt.md](create-prompt.md) structure):
   - **Goal** -- a single sentence stating what the session should accomplish
   - **Context** -- codebase details, architecture, file paths, patterns, prior decisions
   - **Requirements** -- numbered list of specific requirements and constraints
   - **Anti-requirements** -- what NOT to do, common mistakes to avoid
   - **Output** -- what the result should look like: files to create/modify, format, scope

   Flag missing sections that would materially benefit the prompt. Not every prompt needs all five sections -- only flag a section as missing if its absence leaves the prompt ambiguous or underspecified.

3. **Check content quality**:
   Evaluate against these criteria:
   - No common knowledge or vague guidance ("write clean code", "follow best practices", "handle errors properly")
   - No information already covered by the project's CLAUDE.md or rules files (check if a codebase path is available)
   - Specific file paths and function names where applicable, not vague references ("the handler", "the config")
   - Imperative voice throughout ("Add a function...", not "A function should be added...")
   - Under ~60 lines total
   - ASCII-only: no em dashes, smart quotes, curly quotes, or ellipsis characters (these corrupt when pasted across sessions)

4. **Present findings**:
   Group by severity:

   **Blocking** (prompt will underperform without these fixes):
   - Missing Goal section or unclear objective
   - Vague references where specific paths/names are available
   - Common knowledge that wastes context window

   **Improvements** (prompt works but could be sharper):
   - Missing sections that would add value
   - Redundant or verbose phrasing that could be tightened
   - Passive voice or non-imperative phrasing

   **Suggestions** (minor polish):
   - Reordering for clarity
   - Better section titles
   - Non-ASCII characters to replace

   For each finding, state the issue, quote the problematic text, and provide a specific fix.

5. **Offer to rewrite**:
   - MUST ask the user before rewriting: "Want me to apply these fixes?"
   - If approved, apply all fixes and deliver the improved prompt
   - MUST print the improved prompt inside a markdown code block
   - MUST copy the improved prompt to the clipboard via `pbcopy`
   - MUST scan the final prompt for non-ASCII characters and replace with ASCII equivalents before printing or copying
   - MUST tell the user the prompt is copied and ready to paste into a new session
