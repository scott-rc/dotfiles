# Multi-Perspective Review

Three-agent review loop for evaluating skill and rules file quality from complementary angles.

## The Pattern

After applying changes, spawn three review agents in parallel with distinct perspectives:

- **Sonnet (practical executor)** — "As Claude executing these instructions, do they make sense?" Checks checklist compliance, contradictions between files, tier assignments, and under- or over-explained areas. Type: `skill-reviewer` or `rules-reviewer`. Model: `sonnet`.
- **Opus (deep reasoning)** — "Evaluate internal consistency." Checks principle interactions, missing guidance, and edge cases where rules contradict. Type: `skill-reviewer` or `rules-reviewer`. Model: `opus`.
- **Haiku (minimalist)** — "What can still be cut?" Checks token efficiency, redundant items, over-specification, and content that teaches Claude what it already knows. Type: `skill-reviewer` or `rules-reviewer`. Model: `haiku`.

## The Loop

The loop runs until all three agents pass or 4 cycles complete. Each cycle: all three agents review in parallel, findings are synthesized into Blocking / Improvements / Suggestions tiers, Blocking and Improvements issues are fixed, then all three agents re-review the updated files. Suggestions: fix if quick (fewer than 3 per file), otherwise note and move on. If 4 cycles complete without all agents passing, present remaining findings to the user with "acknowledged, not addressed" status and let the user decide.

**Fix delegation**: MUST use `skill-writer` (update mode) for skill file fixes and `rules-writer` for rules file fixes. MUST NOT use `code-writer` — it has no skill/rules validation and its code workflows do not apply to markdown files. Pass the review findings as the problem; let the writer agent determine the implementation.

## Pass Criteria

- All three agents report no Blocking issues.
- All three agents report no Improvements issues, OR the orchestrator judges a flagged item as a design choice rather than a spec violation and explains why.
- Suggestions do not block a pass.

## Handling Disagreements

- If one agent flags something that another agent's perspective makes irrelevant (e.g., Haiku wants to cut content that Opus says is needed for principle consistency), the higher-reasoning perspective wins. Exception: efficiency findings from Haiku that do not affect correctness or consistency SHOULD be accepted unless Opus provides a specific justification for the content's necessity.
- If an agent flags the same item across two or more cycles after an intentional decision not to fix it, record it as "acknowledged, not addressed" with a rationale.

## Agent Prompt Templates

Operations customize these fragments with file paths and skill names.

**Sonnet prompt:**
```
Read [skill-spec.md], [quality-checklist.md], then read all files in <target>. You are Claude executing these instructions. Evaluate: checklist compliance, contradictions between files, correct tier assignments, under-explained or over-explained areas. Quote file names and line numbers. Keep output under 1000 words (500 words on final passes). If no issues, say PASS.
```

**Opus prompt:**
```
Read [skill-spec.md], [quality-checklist.md], then read all files in <target>. Evaluate internal consistency: principle interactions, missing guidance, edge cases where rules contradict or leave Claude without a clear path. Quote file names and line numbers. Keep output under 1000 words (500 words on final passes). If no issues, say PASS.
```

**Haiku prompt:**
```
Read [skill-spec.md], [quality-checklist.md], then read all files in <target>. Evaluate token efficiency: what can be cut without causing mistakes? Flag redundant items, over-specification, and content that teaches Claude what it already knows. Quote file names and line numbers. Keep output under 1000 words (500 words on final passes). If no issues, say PASS.
```
