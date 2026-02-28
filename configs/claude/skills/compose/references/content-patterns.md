# Content Patterns

Reusable patterns for structuring operation steps, task skills, and dynamic context injection.

## Content Patterns for Operation Steps

Use these patterns inside operation files when they fit the task:

- **Checklist pattern**: Give Claude a checklist to copy and track progress through multi-step work. Useful when steps can partially succeed.
- **Feedback loop pattern**: Run validator/linter/tests, fix errors, repeat until clean. A single-evaluator instance of the general loop pattern. Dramatically improves output quality for code-generation or formatting tasks.
- **Template pattern**: Provide a strict output template (low freedom) or a flexible one with optional sections (medium freedom).
- **Examples pattern**: Show 1-2 input/output pairs when the desired style or format is ambiguous.
- **Conditional routing pattern**: "If X, go to step N. If Y, go to step M." Use when an operation has meaningfully different paths.
- **Interview pattern**: When an operation needs user input before proceeding, batch related questions into a single step rather than asking one at a time. Ask only what's needed to unblock the next decision -- don't front-load every possible question. Use follow-up rounds to drill into areas the user's answers reveal as complex or ambiguous. End the interview by summarizing your understanding and confirming before proceeding. Use this pattern when requirements are unclear, the domain has many valid options, or the user's initial request is vague. See the Interview Pattern rules below for AskUserQuestion mechanics.
- **Confirmation pattern**: When asking the user to choose or provide a value, present 1-3 idiomatic defaults as AskUserQuestion options. The tool provides an "Other" free-text option automatically (observed AskUserQuestion behavior — if this changes, callers should add an explicit "Other" option). Choose defaults that reflect common, recommended, or contextually inferred choices -- one click vs typing from scratch. Use for naming choices, location picks, configuration options, disambiguation, and action selection. Do not use for open-ended questions where defaults would mislead, or pure summary confirmations ("does this look right?").
- **Deciding vs doing pattern**: See Delegation > Behavior in the global CLAUDE.md. Quick tests for operation authors — inline if "am I gathering info to decide what to do next?" Delegate if "does this consume context I'll need later?"
- **Scripts vs agents pattern**: Scripts handle deterministic data extraction (fetch, parse, pipe — output is structured data). Agents handle work requiring judgment (triage, analysis, classification, writing — output is a decision or artifact). One-off work stays inline. Work reused 2+ times in the same skill gets extracted to `scripts/` (scripts) or a named agent (agents). Full workflows from another skill use the Skill tool.
- **Cross-skill delegation pattern**: When an operation needs functionality from another skill, use the Skill tool (`skill: "<name>", args: "<routing context>"`). MUST NOT reference another skill's files via relative paths — the other skill's routing and transitive references are not loaded.
- **Subagent delegation pattern**: Subagents get their own context window, reduce orchestrator load, and enable cheaper models. When writing a delegation step, include all context the subagent needs for autonomous work (file paths, conventions, criteria, examples). Choose the right type: `Explore` for search/read, `general-purpose` for multi-step work. Choose the right model: `haiku` for straightforward reads, `sonnet` for analysis and writing, `opus` only for deep reasoning. Subagents return concise results — not raw file contents.
- **Multi-perspective review pattern**: A multi-evaluator instance of the general loop pattern, running Sonnet/practical, Opus/consistency, and Haiku/efficiency reviewers in parallel. See references/multi-perspective-review.md for the full loop mechanics, pass criteria, and prompt templates.

### Interview Pattern

When an operation begins with user Q&A:
- MUST use AskUserQuestion for all questions
- MUST batch related questions into a single AskUserQuestion call (up to 4 questions per call)
- MUST skip questions the user's initial request already answered
- SHOULD complete the interview in 2 AskUserQuestion rounds (8 questions max). A 3rd round is acceptable when the skill has 5+ distinct topics or when follow-up reveals unexpected complexity. MUST NOT exceed 3 rounds.

## Task Skill Pattern

Skills that run a specific workflow (deploys, migrations, data transforms) rather than augmenting knowledge. Combine `context: fork` for isolation with `disable-model-invocation: true` for safety.

A `context: fork` skill runs in an isolated subagent. AskUserQuestion is NOT available inside the fork — any user interaction MUST happen before forking. If the skill needs interactive input, resolve it inline in SKILL.md (where AskUserQuestion works), then dispatch the resolved input to the fork via the Task tool.

```markdown
---
name: run-migration
description: Runs database migrations against the target environment when the user asks to migrate, apply migrations, or update the schema.
disable-model-invocation: true
---

# Run Migration

Run pending database migrations for $ARGUMENTS.

1. **Check current state**: Run `db migrate status` and report pending migrations
2. **Confirm with user**: List migrations that will run and confirm via AskUserQuestion
3. **Apply**: Dispatch migration to a Task subagent (general-purpose): run `db migrate up`, verify with `db migrate status`, report results
```

## Delivery Pattern

When the final output is a prompt or text artifact for the user to paste into another session:

1. Scan for non-ASCII characters and replace with ASCII equivalents ("--" for em dashes, "->" for arrows, straight quotes for smart quotes, "..." for ellipses). Non-ASCII corrupts into mojibake when pasted across sessions.
2. Print the result inside a markdown code block.
3. Copy to the clipboard via `pbcopy`.
4. Tell the user the content is copied and ready to paste.

## Dynamic Context Pattern

Use `` !`command` `` to inject runtime data into skill content. The command runs before content reaches Claude.

```markdown
---
name: review-changes
description: Reviews staged git changes when the user asks to review, check, or inspect their changes.
---

# Review Changes

Review the following staged changes:

!`git diff --cached`

Evaluate for correctness, style, and potential bugs.
```

## Agent Patterns

- **Named agent pattern**: When the same delegation prompt appears in two or more places, extract it into a named agent (`.claude/agents/<name>.md`). The agent file owns the system prompt, tool restrictions, and model selection -- callers send only task-specific context. This is strictly better than duplicating prompts: the instructions exist once, improve in one place, and every caller benefits. The committer agent (`configs/claude/agents/committer.md`) is a good example -- commit, amend, squash, and watch operations all delegate to a single agent that owns commit message rules, UTF-8 handling, and shell safety. Use ad-hoc Task prompts only for truly one-off delegations with no reuse.
- **Companion agent pattern**: Skills that perform complex delegated work SHOULD ship companion agent files at `configs/claude/agents/<agent-name>.md`. The skill operation sends only task-specific details as the task body -- not the full system prompt. This separates stable agent identity (system prompt, tools, model) from variable task content.
- **Memory-enabled agent pattern**: Set `memory: user`, `memory: project`, or `memory: local` in an agent file's frontmatter to enable persistent memory across sessions. Use `memory: user` for personal preferences and cross-project knowledge; `memory: project` for repo-specific conventions the agent should accumulate; `memory: local` for machine-specific state. Include explicit memory management instructions in the agent system prompt: what categories of knowledge to store, when to update entries, and when to prune stale facts.
