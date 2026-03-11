# Review Skill

Evaluate a Claude Code skill against best practices using multi-perspective review, report findings grouped by severity, and fix issues via the review-fix loop until all agents pass.

## Instructions

1. **Locate the skill**:
   - If the user provides a path, use it directly
   - If the user provides a skill name, search for `<name>/SKILL.md` in `~/.claude/skills/` and the project's skill directory
   - If a skill was just created or modified in the current conversation, default to reviewing that skill without asking
   - If neither, discover available skills and present them as AskUserQuestion options
   - Confirm the skill directory exists and contains a SKILL.md file

2. **Multi-perspective review**:
   Spawn 3 Task subagents in parallel (all type: skill-reviewer) per references/multi-perspective-review.md:

   - **Sonnet** — checklist compliance: structure validation, required fields, file links, anti-patterns from the checklist
   - **Opus** — principle consistency: progressive disclosure, workflow quality, degrees of freedom, cross-file coherence
   - **Haiku** — token efficiency: redundancy, over-explaining, tight prose, token justification, splitting candidates

   Each agent receives the skill directory path and its lens as focus. Each returns findings grouped by Blocking/Improvements/Suggestions with per-file token counts. Agent outputs MUST be kept concise — the word limits in prompt templates (e.g., "under 1000 words") are binding. After receiving agent results, summarize each to key findings only before proceeding; do NOT carry full agent transcripts forward into subsequent steps.

3. **Synthesize findings**:
   Merge results from all 3 agents into a single deduplicated list grouped by severity (Blocking > Improvements > Suggestions). Where agents disagree on severity, note the disagreement and use the higher severity. Cross-reference findings against project-specific context the agents would not have (CLAUDE.md conventions, skill interdependencies). Summarize concisely — the synthesized findings list is what persists in context for the fix loop. Drop prose, keep only the structured `file:line — severity — description` format.

4. **Estimate token usage**:
   - Use the token counts from the agents' output
   - Flag individual files over 2000 tokens as candidates for splitting
   - Flag SKILL.md over 5000 tokens as exceeding the hub size limit
   - If skipped for any reason, explicitly state: "Skipping step 4 — no token counts available."

5. **Verify Alloy spec** (if the skill has a `specs/` directory containing `.als` files):
   Run the verification procedure from references/alloy-verification.md. Add any failures as Blocking findings — include the assertion name and counterexample or conformance mismatch. If the skill has no `specs/` directory, explicitly state: "No Alloy specs found, skipping step 5."

6. **Present findings**:
   Group results by severity (Blocking, Improvements, Suggestions). For each finding, state: what the issue is, which file it's in, what the fix would be.

7. **Review-fix loop**:
   Run the evaluate-fix loop per references/multi-perspective-review.md and the project's loop rules. Delegate fixes to a `skill-writer` subagent (update mode; pass: skill directory path, the synthesized findings from step 3, and the specific files to update). Iterate until all pass or 4 cycles complete. The fix loop is the most critical phase — all prior steps exist to serve it. If context is tight, aggressively summarize prior agent outputs before entering the loop. The loop MUST run; truncation before it starts means the review was incomplete.

8. **Report outcomes**:
   Present a summary of what was reviewed, what was fixed, and what remains (pass/fail, cycle count, unresolved findings with severity and reason).
