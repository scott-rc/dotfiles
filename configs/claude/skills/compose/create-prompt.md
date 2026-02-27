# Create Prompt

Craft a session task prompt interactively, producing a polished prompt ready to paste into a new Claude Code session.

## Instructions

1. **Gather requirements**:
   Follow the interview pattern from [content-patterns.md](references/content-patterns.md). Ask the user via AskUserQuestion about:
   - What goal or outcome the session should accomplish
   - Constraints or boundaries (e.g., don't change public API, stay within one file, use existing patterns)
   - Expected output type -- present as AskUserQuestion options: "New feature", "Refactor", "Bug fix", "Migration", "Documentation", "Other"
   - Whether a relevant codebase exists and where it lives
   - Any prior decisions, context, or failed approaches to include

2. **Explore codebase context** (conditional):
   If a codebase is relevant, spawn a Task subagent (type: Explore, model: haiku) to gather context. The subagent MUST:
   - Identify key file paths, directory structure, and architecture
   - Find naming conventions, build/test commands, and relevant CLAUDE.md content
   - Stay focused on what the task actually needs -- do not map the entire codebase
   - Note specific function names, type signatures, or patterns the prompt should reference
   - Return a concise summary of findings (paths, conventions, commands)

   If no codebase applies, MUST skip this step entirely.

3. **Confirm understanding** (conditional):
   Confirm when any requirement was inferred or when multiple valid interpretations exist. Skip when the user's request already stated the goal, constraints, and output clearly.
   When this step runs:
   - MUST summarize the goal, constraints, and planned context in 2-3 sentences
   - MUST present the summary and ask for confirmation via AskUserQuestion with options: "Looks good", "Needs changes" (description: "I'll describe what to adjust"), "Start over" (description: "Re-gather requirements from scratch")
   - If the user selects "Needs changes", ask what to adjust via AskUserQuestion, update understanding, and re-confirm with the same options
   - If the user selects "Start over", return to step 1
   - MUST NOT proceed to drafting until the user selects "Looks good"

4. **Draft the prompt**:
   Write the prompt using this structure (omit sections that don't apply):

   - **Goal** -- one sentence stating what the session should accomplish
   - **Context** -- relevant codebase details: architecture, file paths, patterns, prior decisions
   - **Requirements** -- numbered list of specific requirements and constraints
   - **Anti-requirements** -- what NOT to do, common mistakes to avoid
   - **Output** -- what the result should look like: files to create/modify, format, scope

   Style rules:
   - MUST use imperative voice ("Add a function...", "Modify the handler...")
   - MUST include specific file paths and function names where known
   - SHOULD keep the prompt under ~60 lines
   - MUST NOT include common knowledge or general best practices Claude already knows

5. **Review and tighten**:
   - MUST cut common knowledge (e.g., "write clean code", "handle errors")
   - MUST cut vague guidance (e.g., "follow best practices", "keep it simple")
   - MUST cut anything already covered by the project's CLAUDE.md or rules files
   - SHOULD verify that every line adds information Claude would not have without the prompt

6. **Deliver**:
   MUST follow the delivery pattern from [content-patterns.md](references/content-patterns.md). The deliverable is the final prompt.
