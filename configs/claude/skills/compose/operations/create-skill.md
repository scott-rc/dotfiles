# Create Skill

Scaffold a new Claude Code skill interactively, producing a complete skill directory with SKILL.md, operation files, and reference files.

## Instructions

1. **Gather requirements**:
   Follow the interview pattern from references/content-patterns.md. Cover each topic via AskUserQuestion:
   - Purpose and domain
   - Trigger phrases
   - Operations needed
   - Shared knowledge requirements
   - Runtime dependencies
   - Location — present applicable locations as AskUserQuestion options
   - Invocation mode — present as AskUserQuestion options: "User only" (`disable-model-invocation: true`), "Claude only" (`user-invocable: false`), "Both" (default)
   - Subagent execution (`context: fork`) — present as AskUserQuestion options
   - Tool restrictions (`allowed-tools`)

2. **Determine skill name**:
   - MUST apply naming rules from references/skill-spec.md: lowercase, hyphens, max 64 chars
   - SHOULD prefer gerund form when natural (e.g., `managing-deploys`)
   - MUST confirm the name with the user -- suggest 1-3 name candidates derived from the requirements via AskUserQuestion

3. **Create the skill directory**:
   - Create `<location>/<skill-name>/`
   - If the directory already exists, present options via AskUserQuestion: "Overwrite existing", "Pick a different name"

4. **Confirm the spec**:
   Before writing, confirm the full skill spec with the user. MUST summarize:
   - Skill name and directory path
   - Description (trigger keywords)
   - Operations to create (names and one-line summaries)
   - Reference files to create
   - Frontmatter options (invocation mode, context, allowed-tools)

   Present via AskUserQuestion with options: "Looks good", "Needs changes" (description: "I'll describe what to adjust"), "Start over" (description: "Re-gather requirements from scratch")
   - If "Needs changes", ask what to adjust, update, and re-confirm
   - If "Start over", return to step 1
   - MUST NOT proceed to writing until the user selects "Looks good"

5. **Write the skill**:
   Read the authoring specs (references/skill-spec.md, references/skill-template.md, references/shared-rules.md) and write the skill files inline:
   - Create `<skill_dir>/` via `mkdir -p`
   - Write SKILL.md following the template: frontmatter (`name` matching directory, `description` per spec), Operations section (one H3 per operation), Combined Operations, References
   - Write one `.md` per operation following the operation file template
   - Write one `.md` per reference file with descriptive names
   - Validate against references/quality-checklist.md; fix any issues, up to 3 iterations

6. **Verify Alloy spec** (if the skill has a `specs/` directory containing `.als` files):
   Run the verification procedure from references/alloy-verification.md. Fix any failures before proceeding.
   Note: newly created skills will not have a specs/ directory unless Alloy spec files were manually added. This step applies primarily when creating skills with pre-existing behavioral specifications.

7. **Review and iterate**:
   Run the multi-perspective review loop per references/multi-perspective-review.md. Iterate until all 3 agents pass or 4 cycles complete.

8. **Report results**:
   - MUST list all files created with a one-line description of each
   - MUST show the full `description` field so the user can verify trigger keywords
