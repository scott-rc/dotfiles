# Multi-Perspective Review

Two-agent review loop for evaluating skill and rules file quality from complementary angles. This is a multi-evaluator instance of the evaluate-fix loop pattern, specialized for skill and rules file quality.

## The Pattern

After applying changes, spawn two review agents in parallel with distinct perspectives:

- **Sonnet (practical executor)** — "As Claude executing these instructions, do they make sense?" Checks checklist compliance, contradictions between files, tier assignments, and under- or over-explained areas. Model: `sonnet`.
- **Opus (deep reasoning)** — "Evaluate internal consistency." Checks principle interactions, missing guidance, and edge cases where rules contradict. Model: `opus`.

## The Loop

The loop runs until both agents pass or 4 cycles complete. Each cycle: both agents review in parallel, findings are synthesized into Blocking / Improvement / Suggestion tiers, Blocking and Improvement issues are fixed, then both agents re-review the updated files. Suggestions: fix if quick (fewer than 3 per file), otherwise note and move on.

## Termination

- **Converged** — both agents PASS. Report and exit.
- **No progress** — an iteration produces the same set of unresolved findings as the prior iteration (same files, same severities). Halt, present findings with "acknowledged, not addressed" status, let the user decide.
- **Regression** — findings increase in count or severity after a fix attempt. Halt, present findings, let the user decide.
- **Cycle budget exhausted** — 4 cycles completed without both agents passing. Same as No progress.

**Fix delegation**: Apply all fixes inline using Edit/Write. See the Delegation section in SKILL.md for constraints.

## Pass Criteria

- Both agents report no Blocking issues.
- Both agents report no Improvement issues, OR the orchestrator judges a flagged item as a design choice rather than a spec violation and explains why.
- Suggestions do not block a pass.

## Handling Disagreements

- When Sonnet and Opus disagree, the higher-reasoning perspective (Opus) wins on consistency and correctness matters. Sonnet's practical execution perspective wins on "does this actually work when executed" matters.
- If an agent flags the same item across two or more cycles after an intentional decision not to fix it, record it as "acknowledged, not addressed" with a rationale.

## Agent Prompt Templates

Operations customize `<spec-file>` and `<target>` when dispatching. `<spec-file>` is the compose spec that matches the artifact being reviewed (`configs/claude/skills/compose/references/skill-spec.md` for skills, `configs/claude/skills/compose/references/rules-spec.md` for rules files). `<target>` is the directory or file under review.

**Sonnet prompt:**
```
Read <spec-file> and configs/claude/skills/compose/references/quality-checklist.md, then read all files in <target>. You are Claude executing these instructions. Evaluate: checklist compliance, contradictions between files, correct tier assignments, under-explained or over-explained areas. Quote file names and line numbers. Format each finding as `file:line — severity — one sentence`. Keep output under 1000 words (500 words on final passes). If no issues, say PASS.
```

**Opus prompt:**
```
Read <spec-file> and configs/claude/skills/compose/references/quality-checklist.md, then read all files in <target>. Evaluate internal consistency: principle interactions, missing guidance, edge cases where rules contradict or leave Claude without a clear path. Quote file names and line numbers. Format each finding as `file:line — severity — one sentence`. Keep output under 1000 words (500 words on final passes). If no issues, say PASS.
```
