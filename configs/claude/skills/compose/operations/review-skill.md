# Review Skill

Evaluate a Claude Code skill against best practices using multi-perspective review, report findings grouped by severity, and fix issues via the review-fix loop until all agents pass.

## Instructions

1. **Locate the skill**:
   - If the user provides a path, use it directly
   - If the user provides a skill name, search for `<name>/SKILL.md` in `~/.claude/skills/` and the project's skill directory
   - If a skill was just created or modified in the current conversation, default to reviewing that skill without asking
   - If neither, discover available skills and present them as options
   - Confirm the skill directory exists and contains a SKILL.md file

2. **Multi-perspective review**:
   Spawn 2 subagents in parallel per references/multi-perspective-review.md, substituting `<spec-file>` = `configs/claude/skills/compose/references/skill-spec.md` and `<target>` = the skill directory under review.

   - **Sonnet** — checklist compliance: structure validation, required fields, file links, anti-patterns from the checklist
   - **Opus** — principle consistency: progressive disclosure, workflow quality, degrees of freedom, cross-file coherence

   Each agent receives the target directory path and its lens as focus. Each returns findings tagged with severity (Blocking / Improvement / Suggestion). Agent outputs MUST be kept concise — the word limits in prompt templates (e.g., "under 1000 words") are binding. After receiving agent results, summarize each to key findings only before proceeding; do NOT carry full agent transcripts forward into subsequent steps.

3. **Synthesize findings**:
   Merge results from both agents into a single deduplicated list grouped by severity (Blocking > Improvement > Suggestion). Where agents disagree on severity, note the disagreement and use the higher severity. Cross-reference findings against project-specific context the agents would not have (CLAUDE.md conventions, skill interdependencies). Summarize concisely — the synthesized findings list is what persists in context for the fix loop. Drop prose, keep only the structured `file:line — severity — description` format.

4. **Present findings**:
   Group results by severity (Blocking, Improvement, Suggestion). For each finding, state: what the issue is, which file it's in, what the fix would be.

5. **Review-fix loop**:
   Run the loop per references/multi-perspective-review.md. Apply fixes inline using Edit/Write — read the authoring specs (references/skill-spec.md, references/skill-template.md) if needed for guidance. Iterate until the loop terminates per the Termination section in multi-perspective-review.md. The fix loop is the most critical phase — all prior steps exist to serve it. If context is tight, aggressively summarize prior agent outputs before entering the loop. The loop MUST run; truncation before it starts means the review was incomplete.

6. **Report outcomes**:
   Present a summary of what was reviewed, what was fixed, and what remains (pass/fail, cycle count, unresolved findings with severity and reason).
