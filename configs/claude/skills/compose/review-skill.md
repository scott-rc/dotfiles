# Review Skill

Evaluate a Claude Code skill against best practices, report findings grouped by severity, and offer to fix issues.

## Instructions

1. **Locate the skill**:
   - If the user provides a path, use it directly
   - If the user provides a skill name, search for `<name>/SKILL.md` in `~/.claude/skills/` and the project's skill directory
   - If neither, discover available skills and present them as AskUserQuestion options
   - Confirm the skill directory exists and contains a SKILL.md file

2. **Read all skill files via subagent**:
   Spawn a Task subagent (type: Explore, model: haiku) to read the skill directory. The subagent MUST:
   - Read SKILL.md first
   - Read every `.md` file linked from SKILL.md (operations and references)
   - Check for orphan `.md` files in the directory not linked from anywhere
   - Read any scripts in `scripts/` if present
   - Return a structured summary: for each file, its path, approximate token count (1 token per 4 chars), linked files, and a 1-2 sentence content summary

3. **Validate structure against spec**:
   MUST validate the skill against every rule in [skill-spec.md](skill-spec.md) and [shared-rules.md](shared-rules.md), covering frontmatter, naming, SKILL.md body, operation files, reference files, and orphan files. Use the subagent's summary from step 2 -- only read individual files inline when a finding needs verification.

4. **Evaluate content quality against checklist**:
   MUST evaluate against every item in [quality-checklist.md](quality-checklist.md).

5. **Check for additional anti-patterns** not covered by the checklist:
   - Operation files that duplicate content from other operation files
   - Reference files that contain operation logic (numbered steps telling Claude what to do)
   - Missing combined operations when multiple operations could logically be chained
   - Skills with side effects that don't use `disable-model-invocation: true`
   - Skills with `context: fork` that contain only reference content (no task instructions)
   - Skills with long descriptions that may exceed the description budget (2% of context window)
   - Operations that read many files or explore codebases inline instead of delegating to subagents

6. **Estimate token usage**:
   - Use the token counts from the subagent's summary in step 2
   - Flag files over 2000 tokens as candidates for splitting
   - Flag total skill size over 5000 tokens as potentially too large for SKILL.md

7. **Present findings**:
   Group results by severity:

   **Blocking** (MUST fix):
   - Missing required frontmatter fields
   - Broken file links
   - Missing operation files
   - Anti-patterns from the checklist

   **Improvements** (SHOULD fix):
   - Vague description lacking trigger keywords
   - Missing error handling in operations
   - Redundant content between files
   - Missing combined operations section

   **Suggestions** (MAY fix):
   - Better file naming
   - Additional examples
   - Token optimization opportunities

   For each finding, state:
   - What the issue is
   - Which file it's in
   - What the fix would be (specific, not vague)

8. **Offer to apply fixes**:
   - MUST ask the user about blocking fixes before applying them
   - SHOULD present improvements and suggestions as AskUserQuestion options for the user to select
   - MUST apply fixes one at a time, confirming each change
