---
name: evidence
description: Verifies claims and assertions the agent just made by gathering evidence from the codebase and runtime — use when the user asks to back up, cite, source, verify, substantiate, support, or prove something that was just said.
---

# Evidence

Retrospectively verify claims or assertions made in recent conversation turns by gathering evidence for each.

## Input Resolution

- **Empty / no arguments** — Scan the recent conversation to identify claims, assertions, or factual statements the agent made. A claim is any statement that could be verified or refuted with evidence — not opinions, questions, or hedged speculation. Present the identified claims via AskUserQuestion and ask which to verify (default: all).
- **Plain text** — Treat as a specific claim to verify. Skip claim extraction and proceed directly to Verify.

After resolving, dispatch to Verify via the Task tool (`subagent_type: general-purpose`). Pass the resolved claim(s) and the operation file path as context.

## Operations

### Verify
Gathers supporting, contradicting, and inconclusive evidence for one or more claims and writes results to `tmp/evidence-<slug>.md`.
See operations/verify.md for detailed instructions.

## Combined Operations

- **"back up"** / **"cite"** / **"source"** / **"verify"** / **"substantiate"** / **"support"** / **"prove"** / **"evidence"** → Verify
- **No arguments** → Extract claims from conversation, confirm with user, then Verify
