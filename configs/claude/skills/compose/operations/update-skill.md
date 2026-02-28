# Update Skill

Modify an existing Claude Code skill by adding, changing, or removing operations, reference files, and SKILL.md content.

## Instructions

1. **Locate the skill**:
   - If the user provides a path, use it directly
   - If the user provides a skill name, search for `<name>/SKILL.md` in `~/.claude/skills/` and the project's `.claude/skills/` directory
   - If neither, discover available skills and present them as AskUserQuestion options
   - Confirm the skill directory exists and contains a SKILL.md file

2. **Read the current skill**:
   Spawn a Task subagent (type: Explore, model: haiku) to read all files in the skill directory. The subagent MUST return a structured summary (not raw file contents):
   - Frontmatter fields (name, description, argument-hint, etc.)
   - Operation names with their one-line summaries
   - Combined Operations entries
   - Reference file names with their topics
   - Per-file line counts

3. **Check for Alloy spec**:
   Check if the skill has a `specs/` directory containing `.als` files. If found, read the spec to understand the skill's behavioral invariants (operations, routing, delegation, step sequences). These invariants constrain what changes are valid — use them to inform step 4 and verify in step 6.

4. **Determine the update scope**:
   If the user's request is specific (e.g., "add a deploy operation"), proceed directly. If the request is vague (e.g., "update this skill"), interview the user via AskUserQuestion about what to change. Present the current skill structure from step 2 and ask which of these update types apply:
   - Add a new operation
   - Modify an existing operation
   - Remove an operation
   - Add or update a reference file
   - Update SKILL.md (description, combined operations, frontmatter)
   - Remove a reference file
   - Rename the skill

   Multiple update types may apply simultaneously. Confirm all planned changes together via AskUserQuestion before proceeding. Apply them in the order listed in step 5.

5. **Apply changes**:
   Spawn a Task subagent (type: skill-writer) in update mode. Pass:
   - `mode`: update
   - `skill_dir`: the skill directory path from step 1
   - `spec`: any new requirements (name, description, operations, references, frontmatter options)
   - `update_scope`: the confirmed changes from step 4
   - `existing_summary`: the structure summary from step 2

   The skill-writer reads authoring specs, applies changes (add/modify/remove operations, add/update/remove references, update metadata, rename), validates against the quality checklist, and self-corrects up to 3 iterations. It returns the list of files created/modified/removed, validation status, and per-file token counts.

   Special cases requiring user interaction before delegation:
   - **Renaming**: confirm new name candidates via AskUserQuestion before passing to skill-writer
   - **Removing references**: if operation files link to a reference being removed, warn the user and confirm via AskUserQuestion before proceeding
   - **Orphaned references**: if removing an operation orphans reference files, present via AskUserQuestion: "Remove orphaned file", "Keep it"

   MUST fix any blocking issues the skill-writer reports before proceeding.

6. **Verify Alloy spec** (if spec exists from step 3):
   Run `alloy exec -f -o /tmp/alloy-output <spec-path>` for each `.als` file. If any check returns SAT (counterexample found), the update violates a behavioral invariant. Read the Alloy output to identify which assertion failed, map it back to the change, and fix the skill files or the spec before proceeding. Re-run until all checks return UNSAT.

7. **Review and iterate**:
   Run the multi-perspective review loop per references/multi-perspective-review.md. Iterate until all 3 agents pass or 4 cycles complete.

8. **Report results**:
   - MUST list all files added, modified, or removed with a one-line description of each change
   - MUST show the updated SKILL.md Operations section so the user can verify
   - If the description changed, show the full `description` field so the user can verify trigger keywords
   - If the skill was renamed, MUST show the old and new directory paths
   - MUST note any blocking findings from the skill-writer's validation and how they were resolved, if any were found
   - MUST include the final review status (pass/fail, number of cycles, any acknowledged-but-not-fixed items)
