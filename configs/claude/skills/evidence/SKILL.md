---
name: evidence
description: Investigates questions and builds structured arguments by collecting proof from code, runtime, and research sources — use when the user asks to investigate, gather evidence, prove, disprove, or verify a claim.
disable-model-invocation: true
argument-hint: "[question | claim | URL | session-id]"
---

# Evidence

Investigate a question or build a structured case for or against a claim by collecting evidence from code, runtime, and research sources.

## Input Resolution

Resolve $ARGUMENTS to a plain-text question or claim before dispatching.

- **Empty** — Use AskUserQuestion to ask: "What would you like me to investigate or evaluate?"
- **UUID-like string** — Run `claude session read <id>`. Scan the transcript for main topics. Use AskUserQuestion to present the topics found and ask which to focus on.
- **GitHub URL** — Fetch content via `gh api` (parse URL to construct endpoint). Extract the concern or question raised.
- **Slack URL or reference** — Fetch via `claude_ai_Slack:slack_read_thread` or `claude_ai_Slack:slack_read_channel`. Extract the concern or question.
- **Plain text** — Use directly as the resolved input.

After resolving, dispatch to the appropriate operation via the Task tool (`subagent_type: general-purpose`). Pass the resolved question or claim and the operation file path as context.

## Operations

Each operation runs in a subagent after input is resolved inline above.

### Investigate
Open-ended research of a question or topic; collects findings and writes them to `tmp/evidence-<slug>.md`.
See operations/investigate.md for detailed instructions.

### Build Case
Structured argument for or against a claim; collects supporting and contradicting evidence and writes to `tmp/case-<slug>.md`.
See operations/build-case.md for detailed instructions.

## Combined Operations

- **"investigate"** / **"look into"** / **"what evidence"** / **"find proof"** → Investigate
- **"build a case"** / **"prove"** / **"disprove"** / **"argue"** / **"make the case"** → Build Case
- **"verify"** / **"check if"** → Build Case (treat the statement as the claim)
- If ambiguous whether question or claim → default to Investigate
