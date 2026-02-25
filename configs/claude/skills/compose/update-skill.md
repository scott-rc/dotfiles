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

3. **Determine the update scope**:
   If the user's request is specific (e.g., "add a deploy operation"), proceed directly. If the request is vague (e.g., "update this skill"), interview the user via AskUserQuestion about what to change. Present the current skill structure from step 2 and ask which of these update types apply:
   - Add a new operation
   - Modify an existing operation
   - Remove an operation
   - Add or update a reference file
   - Update SKILL.md (description, combined operations, frontmatter)
   - Remove a reference file
   - Rename the skill

   Multiple update types may apply simultaneously. Confirm all planned changes together via AskUserQuestion before proceeding. Apply them in the order listed in step 4.

4. **Apply changes**:
   For each change in the confirmed scope, apply in this order:

   **Adding an operation**:
   - MUST read [skill-spec.md](skill-spec.md) and [skill-template.md](skill-template.md) before writing
   - MUST read [shared-rules.md](shared-rules.md) and apply Content Rules
   - Write the operation file following the operation file template
   - Add the operation's H3 entry to SKILL.md's Operations section
   - Update Combined Operations if the new operation produces output that another operation consumes, or if users would naturally request both together. Otherwise, leave Combined Operations unchanged.
   - Add any new reference files the operation needs
   - After writing, verify: operation file H1 matches SKILL.md operation name, all file links resolve

   **Modifying an operation**:
   - Read the existing operation file. If the file exceeds 100 lines, SHOULD spawn a Task subagent (type: Explore, model: haiku) to read it and return only the sections relevant to the requested change.
   - Apply the requested changes using Edit, preserving the existing structure (H1, summary, numbered steps)
   - If the operation's name or summary changed, update the matching H3 entry in SKILL.md
   - If the operation now references new files, add them to the References section
   - After editing, verify: H1 still matches SKILL.md operation name, all file links resolve

   **Removing an operation**:
   - Delete the operation file
   - Remove its H3 entry from SKILL.md
   - Remove it from Combined Operations entries in SKILL.md
   - Check if any reference files are now orphaned (not referenced by any remaining operation). If so, present orphans via AskUserQuestion: "Remove orphaned file", "Keep it"

   **Adding or updating a reference file**:
   - MUST name files descriptively: `<topic>-<type>.md` (e.g., `deploy-patterns.md`)
   - MUST provide information, not step-by-step instructions
   - Add the file link to SKILL.md's References section if not already listed

   **Removing a reference file**:
   - Check all operation files for links to this reference. If any exist, warn the user and confirm via AskUserQuestion before proceeding.
   - Delete the file
   - Remove its entry from SKILL.md's References section

   **Updating SKILL.md metadata**:
   - Edit frontmatter fields (description, argument-hint, etc.)
   - Update the body text, Combined Operations, or References section as needed
   - MUST keep the `name` field matching the directory name

   **Renaming the skill**:
   - Confirm the new name with the user -- suggest 1-3 candidates via AskUserQuestion
   - MUST apply naming rules from [skill-spec.md](skill-spec.md): lowercase, hyphens, max 64 chars
   - If the target directory already exists, present options via AskUserQuestion: "Pick a different name", "Cancel rename"
   - Rename the directory
   - Update the `name` field in SKILL.md frontmatter
   - Warn the user about any external references (other skills, CLAUDE.md, agents) that may need manual updates

5. **Validate**:
   Spawn a Task subagent (type: skill-reviewer) with the updated skill directory path.
   MUST fix any blocking issues found before reporting to the user. SHOULD present improvements and suggestions to the user via AskUserQuestion before applying.

6. **Report results**:
   - MUST list all files added, modified, or removed with a one-line description of each change
   - MUST show the updated SKILL.md Operations section so the user can verify
   - If the description changed, show the full `description` field so the user can verify trigger keywords
   - If the skill was renamed, MUST show the old and new directory paths
   - MUST note any blocking findings from the skill-reviewer and how they were resolved, if any were found
