# Evaluate-Fix Loops

An evaluate-fix cycle that repeats until convergence. The orchestrator drives; evaluator and fixer agents do the work.

**Structure** (repeat until converged or max iterations reached):

1. **Evaluate** — run evaluator agents in parallel; each finding MUST be `file:line — severity — one sentence`
2. **Fix** — immediately delegate all Blocking and Improvement findings to the appropriate fixer; no confirmation, no pause
3. **Re-evaluate** — run the same evaluators on updated state
4. **Converge** — stop when pass criteria are met or max iterations (default: 4) reached

**Rules:**

- MUST proceed through evaluate → fix → re-evaluate without pausing to ask the user
- **Blocking** / **Improvements** — fix immediately; escalate only when the fix has multiple plausible approaches and no available context disambiguates
- **Recurring findings** — if the same finding recurs after a fix attempt, either escalate to the user or record as "acknowledged, not addressed" with rationale
- **Suggestions** — fix if quick (fewer than 3 per file); otherwise note and move on; do not block convergence
- When max iterations complete without convergence, present remaining findings with status and let the user decide
