# Shared Authoring Rules

Rules that apply to both skills and rules files. All operations in this skill validate against these rules.

## Keyword Conventions

The key words MUST, MUST NOT, SHOULD, SHOULD NOT, and MAY in skill and rules files are used as described in RFC 2119:

- **MUST** / **MUST NOT** — absolute requirement or prohibition. The instruction is broken if violated.
- **SHOULD** / **SHOULD NOT** — strong recommendation. Can be ignored only with good reason.
- **MAY** — truly optional. No justification needed to omit.

Write these keywords in ALL CAPS when used with their RFC meaning. Use lowercase for ordinary prose. Reserve MUST for rules where violation causes failure — overuse dilutes its authority.

## Content Rules

- **Context window is a public good**: MUST only add information Claude does not already have. Challenge each line: does this teach something new, or does it restate common knowledge? Every token MUST justify its cost.
- **Delegate aggressively to subagents**: Operations MUST default to delegating work to Task subagents or named agents. The orchestrator's context is finite and shared across the entire session -- every file read, diff analysis, or verbose result inline is context permanently lost to later work. Subagents get their own context window, can use cheaper/faster models, and can receive tightly composed instructions that make them more likely to do the right thing. The orchestrator gathers requirements, makes decisions requiring user interaction, and synthesizes results. Everything else goes to a subagent. When the same delegation pattern appears in two or more places, extract it into a named agent (`configs/claude/agents/<name>.md`) so the instructions exist once and improve in one place.
- **Write tight**: SHOULD use terse, imperative prose. Drop articles, filler words, and hedging where meaning is preserved. Prefer sentence fragments in lists. Lead with the verb. Example: "MUST run linter before committing" not "You should make sure to run the linter tool before you commit your changes".
- **No time-sensitive information**: MUST NOT reference specific versions, dates, or URLs that will rot
- **Consistent terminology**: MUST pick one term and use it everywhere (e.g., "operation" not sometimes "command" and sometimes "action")
- **POSIX paths**: MUST use forward slashes. No backslashes, no Windows paths.
- **Markdown only**: All skill and rules files MUST be markdown. Use code blocks for shell commands.
- **No tables**: MUST use lists instead of markdown tables. Tables add significant token overhead (pipes, header separators, padding) with no benefit for LLM comprehension. Use bulleted lists with `—` separators for key-value pairs, or split into labeled sub-lists for multi-column data.
- **RFC 2119 keywords**: SHOULD use MUST, SHOULD, and MAY (capitalized) per the Keyword Conventions section above. Reserve MUST for rules where violation breaks the outcome.
