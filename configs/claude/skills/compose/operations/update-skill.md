# Update Skill

Modify an existing Claude Code skill by adding, changing, or removing operations, reference files, and SKILL.md content.

## Instructions

1. **Locate the skill**:
   - If the user provides a path, use it directly
   - If the user provides a skill name, search for `<name>/SKILL.md` in `~/.claude/skills/` and the project's `.claude/skills/` directory
   - If neither, discover available skills and present them as options
   - Confirm the skill directory exists and contains a SKILL.md file

2. **Read the current skill**:
   Read SKILL.md and the files relevant to the planned changes. Note:
   - Frontmatter fields (name, description, argument-hint, etc.)
   - Operation names with their one-line summaries
   - Combined Operations entries
   - Reference file names with their topics

3. **Determine the update scope**:
   If the user's request is specific (e.g., "add a deploy operation"), proceed directly. If the request is vague (e.g., "update this skill"), interview the user about what to change. Present the current skill structure from step 2 and ask which of these update types apply:
   - Add a new operation
   - Modify an existing operation
   - Remove an operation
   - Add or update a reference file
   - Update SKILL.md (description, combined operations, frontmatter)
   - Remove a reference file
   - Rename the skill

   Multiple update types may apply simultaneously. Confirm all planned changes together with the user before proceeding. Apply them in the order listed in step 4.

4. **Apply changes**:
   Read the authoring specs (references/skill-spec.md, references/skill-template.md, references/shared-rules.md) and apply changes inline using Edit/Write:

   - **Adding an operation**: write the operation file following the template, add H3 to SKILL.md Operations, update Combined Operations if applicable
   - **Modifying an operation**: edit the operation file preserving structure (H1, summary, numbered steps), update SKILL.md H3 if name or summary changed
   - **Removing an operation**: delete the file, remove H3 from SKILL.md and Combined Operations, list orphaned references
   - **Adding/updating a reference**: write or edit the file, add to SKILL.md References if not listed
   - **Removing a reference**: delete the file, remove from SKILL.md References, note operation files that linked to it
   - **Updating SKILL.md metadata**: edit frontmatter, body, Combined Operations, or References
   - **Renaming**: rename directory via `mv`, update `name` in frontmatter, note external references needing manual updates

   Special cases requiring user interaction before applying:
   - **Renaming**: confirm new name candidates with the user
   - **Removing references**: if operation files link to a reference being removed, warn and confirm with the user
   - **Orphaned references**: present options to the user: "Remove orphaned file", "Keep it"

   Validate against references/quality-checklist.md; fix any issues, up to 3 iterations.

5. **Review and iterate**:
   Run the multi-perspective review loop per references/multi-perspective-review.md. Iterate until all 3 agents pass or 4 cycles complete.

6. **Report results**:
   - MUST list all files added, modified, or removed with a one-line description of each change
   - MUST show the updated SKILL.md Operations section so the user can verify
   - If the description changed, show the full `description` field so the user can verify trigger keywords
   - If the skill was renamed, MUST show the old and new directory paths
   - MUST note any blocking validation findings and how they were resolved
   - MUST include the final review status (pass/fail, number of cycles, any acknowledged-but-not-fixed items)
