# Review Skill

Evaluate a Claude Code skill against best practices using multi-perspective review, report findings grouped by severity, and iterate fixes until clean.

## Instructions

1. **Locate the skill**:
   - If the user provides a path, use it directly
   - If the user provides a skill name, search for `<name>/SKILL.md` in `~/.claude/skills/` and the project's skill directory
   - If neither, discover available skills and present them as AskUserQuestion options
   - Confirm the skill directory exists and contains a SKILL.md file

2. **Multi-perspective review**:
   Spawn 3 Task subagents in parallel (all type: skill-reviewer) per references/multi-perspective-review.md:

   - **Sonnet** — checklist compliance: structure validation, required fields, file links, anti-patterns from the checklist
   - **Opus** — principle consistency: progressive disclosure, workflow quality, degrees of freedom, cross-file coherence
   - **Haiku** — token efficiency: redundancy, over-explaining, tight prose, token justification, splitting candidates

   Each agent receives the skill directory path and its lens as focus. Each returns findings grouped by Blocking/Improvements/Suggestions with per-file token counts.

3. **Synthesize findings**:
   Merge results from all 3 agents into a single deduplicated list grouped by severity (Blocking > Improvements > Suggestions). Where agents disagree on severity, note the disagreement and use the higher severity. Cross-reference findings against project-specific context the agents would not have (CLAUDE.md conventions, skill interdependencies).

4. **Estimate token usage**:
   - Use the token counts from the agents' output
   - Flag individual files over 2000 tokens as candidates for splitting
   - Flag SKILL.md over 5000 tokens as exceeding the hub size limit

5. **Present findings**:
   Group results by severity (Blocking, Improvements, Suggestions). For each finding, state: what the issue is, which file it's in, what the fix would be.

6. **Review-fix loop**:
   - Fix immediately without pausing to ask the user. Escalate only when a fix has multiple plausible approaches and no available context disambiguates, or the same finding recurs after a fix attempt.
   - Suggestions: fix if quick (fewer than 3 per file); otherwise note and move on. Do not block convergence on Suggestions.
   - Delegate fixes to a `skill-writer` subagent (update mode), then re-review with all 3 agents.
   - Iterate until all findings pass or 4 cycles complete per references/multi-perspective-review.md.

7. **Report outcomes**:
   Present a summary of what was reviewed, what was fixed, and what remains. List any unresolved findings with their severity and the reason they were not fixed (escalated, recurring, or max iterations reached). If all findings were resolved, say so explicitly. MUST include the final review status (pass/fail, number of cycles, any acknowledged-but-not-fixed items).
