# Multi-Perspective Review

Two-agent review loop for evaluating skill and rules file quality from complementary angles. This is a multi-evaluator instance of the evaluate-fix loop pattern, specialized for skill and rules file quality.

## The Pattern

After applying changes, spawn two review agents in parallel with distinct perspectives:

- **Sonnet (practical executor)** — "As Claude executing these instructions, do they make sense?" Checks checklist compliance, contradictions between files, tier assignments, and under- or over-explained areas. Type: `skill-reviewer` or `rules-reviewer`. Model: `sonnet`.
- **Opus (deep reasoning)** — "Evaluate internal consistency." Checks principle interactions, missing guidance, and edge cases where rules contradict. Type: `skill-reviewer` or `rules-reviewer`. Model: `opus`.

## The Loop

The loop runs until both agents pass or 4 cycles complete. Each cycle: both agents review in parallel, findings are synthesized into Blocking / Improvements / Suggestions tiers, Blocking and Improvements issues are fixed, then both agents re-review the updated files. Suggestions: fix if quick (fewer than 3 per file), otherwise note and move on. If 4 cycles complete without both agents passing, present remaining findings to the user with "acknowledged, not addressed" status and let the user decide.

**Fix delegation**: Apply skill file fixes inline using Edit/Write. Use `rules-writer` for rules file fixes. See the Delegation section in SKILL.md for constraints.

## Pass Criteria

- Both agents report no Blocking issues.
- Both agents report no Improvements issues, OR the orchestrator judges a flagged item as a design choice rather than a spec violation and explains why.
- Suggestions do not block a pass.

## Handling Disagreements

- When Sonnet and Opus disagree, the higher-reasoning perspective (Opus) wins on consistency and correctness matters. Sonnet's practical execution perspective wins on "does this actually work when executed" matters.
- If an agent flags the same item across two or more cycles after an intentional decision not to fix it, record it as "acknowledged, not addressed" with a rationale.

## Agent Prompt Templates

Operations customize these fragments with file paths and skill names.

**Sonnet prompt:**
```
Read references/skill-spec.md, references/quality-checklist.md, then read all files in <target>. (Resolve references/ paths relative to the compose skill directory, not the target being reviewed.) You are Claude executing these instructions. Evaluate: checklist compliance, contradictions between files, correct tier assignments, under-explained or over-explained areas. Quote file names and line numbers. Format each finding as `file:line — severity — one sentence`. Keep output under 1000 words (500 words on final passes). If no issues, say PASS.
```

**Opus prompt:**
```
Read references/skill-spec.md, references/quality-checklist.md, then read all files in <target>. (Resolve references/ paths relative to the compose skill directory, not the target being reviewed.) Evaluate internal consistency: principle interactions, missing guidance, edge cases where rules contradict or leave Claude without a clear path. Quote file names and line numbers. Format each finding as `file:line — severity — one sentence`. Keep output under 1000 words (500 words on final passes). If no issues, say PASS.
```

**For rules reviews**: Use the same two prompt templates above, but substitute `rules-reviewer` as the agent type instead of `skill-reviewer`, and instruct agents to read `references/rules-spec.md` instead of (or in addition to) `references/skill-spec.md`.
