# Prompt Structure

A session task prompt uses up to five sections. Omit sections that don't apply.

- **Goal** -- one sentence stating what the session should accomplish
- **Context** -- relevant codebase details: architecture, file paths, patterns, prior decisions
- **Requirements** -- numbered list of specific requirements and constraints
- **Anti-requirements** -- what NOT to do, common mistakes to avoid
- **Output** -- what the result should look like: files to create/modify, format, scope

## Style Rules

- MUST use imperative voice ("Add a function...", "Modify the handler...")
- MUST include specific file paths and function names where known
- SHOULD keep the prompt under ~60 lines
- MUST NOT include common knowledge or general best practices Claude already knows
