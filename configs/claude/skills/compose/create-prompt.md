# Create Prompt

Craft a session task prompt interactively, producing a polished prompt ready to paste into a new Claude Code session.

## Instructions

1. **Gather requirements**:
   Interview the user about:
   - What goal or outcome the session should accomplish
   - Constraints or boundaries (e.g., don't change public API, stay within one file, use existing patterns)
   - Expected output type (new feature, refactor, bug fix, migration, documentation, etc.)
   - Whether a relevant codebase exists and where it lives
   - Any prior decisions, context, or failed approaches to include

   MUST batch questions into a single message. MUST skip any questions the user's initial request already answered. SHOULD ask follow-ups only if the answers are ambiguous or incomplete.

2. **Explore codebase context** (conditional):
   If a codebase is relevant:
   - MUST explore it for key file paths, directory structure, and architecture
   - SHOULD identify naming conventions, build/test commands, and relevant CLAUDE.md content
   - MUST stay focused on what the task actually needs — do not map the entire codebase
   - SHOULD note specific function names, type signatures, or patterns the prompt should reference

   If no codebase applies, MUST skip this step entirely.

3. **Confirm understanding**:
   - MUST summarize the goal, constraints, and planned context in 2-3 sentences
   - MUST NOT proceed to drafting until the user confirms the summary is accurate
   - If the user corrects anything, update understanding and re-confirm

4. **Draft the prompt**:
   Write the prompt using this structure (MUST omit sections that don't apply):

   - **Goal** -- one sentence stating what the session should accomplish
   - **Context** -- relevant codebase details: architecture, file paths, patterns, prior decisions
   - **Requirements** -- numbered list of specific requirements and constraints
   - **Anti-requirements** -- what NOT to do, common mistakes to avoid
   - **Output** -- what the result should look like: files to create/modify, format, scope

   MUST use imperative voice ("Add a function...", "Modify the handler..."). MUST include specific file paths and function names where known. SHOULD keep the prompt under ~60 lines. MUST NOT include common knowledge or general best practices the model already knows.

5. **Review and tighten**:
   - MUST cut common knowledge (e.g., "write clean code", "handle errors")
   - MUST cut vague guidance (e.g., "follow best practices", "keep it simple")
   - MUST cut anything already covered by the project's CLAUDE.md or rules files
   - SHOULD verify that every line adds information the model would not have without the prompt

6. **Deliver**:
   - MUST scan the final prompt for non-ASCII characters and replace them with ASCII equivalents before printing or copying: "--" for em dashes, "->" for arrows, straight quotes for smart quotes, "..." for ellipses. Non-ASCII characters corrupt into mojibake when pasted across sessions.
   - MUST print the final prompt inside a markdown code block
   - MUST copy the prompt to the clipboard via `pbcopy`
   - MUST tell the user the prompt is copied and ready to paste into a new session
