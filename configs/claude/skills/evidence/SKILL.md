---
name: evidence
description: Verifies claims and assertions the agent just made by gathering evidence from the codebase and runtime — use when the user asks to back up, cite, source, verify, substantiate, support, or prove something that was just said.
---

# Evidence

Retrospectively verify claims or assertions made in recent conversation turns by gathering evidence for each.

## Input Resolution

- **Empty / no arguments** — Scan the recent conversation to identify claims, assertions, or factual statements the agent made. A claim is any statement that could be verified or refuted with evidence — not opinions, questions, or hedged speculation. When claims are unambiguous and ≤10, list them in the response and proceed immediately — the user can interrupt to exclude any. Ask the user only when claims are ambiguous, numerous (>10), or when it's unclear what counts as a claim.
- **Plain text** — Treat as a specific claim to verify. Skip claim extraction and proceed directly to Verify.

After resolving, read operations/verify.md and execute inline — do NOT delegate to a subagent. The orchestrator has the conversation context where claims were made; a subagent does not.

## Operations

### Verify
Gathers supporting, contradicting, and inconclusive evidence for one or more claims and writes results to `tmp/evidence-<slug>.md`.
MUST read operations/verify.md before executing. Execute inline.

## Combined Operations

- **"back up"** / **"cite"** / **"source"** / **"verify"** / **"substantiate"** / **"support"** / **"prove"** / **"evidence"** → Verify
- **No arguments** → Extract claims from conversation, list and proceed (or ask the user if ambiguous/numerous), then Verify
