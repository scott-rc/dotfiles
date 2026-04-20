# Create Skill

Scaffold a new Claude Code skill interactively, producing a complete skill directory with SKILL.md, operation files, and reference files.

## Instructions

1. **Gather requirements**:
   Follow the interview pattern from references/content-patterns.md. Cover each topic:
   - Purpose and domain
   - Trigger phrases
   - Operations needed
   - Shared knowledge requirements
   - Runtime dependencies
   - Location — present applicable locations as options
   - Invocation mode — present as options: "User only" (`disable-model-invocation: true`), "Claude only" (`user-invocable: false`), "Both" (default)
   - Subagent execution (`context: fork`) — present as options
   - Tool restrictions (`allowed-tools`)

2. **Determine skill shape**:
   Based on the gathered requirements, decide whether the skill should be **simple** (single SKILL.md, inline instructions) or **hub-and-spoke** (SKILL.md router + operation files + optional references). Default to simple. Use hub-and-spoke only if at least one of these triggers is met:
   - An operation has conditional branches or multi-path logic
   - An operation reads shared reference files (patterns, guidelines, templates)
   - An operation delegates to a subagent with substantial context
   - There are enough operations that inlining all of them would make SKILL.md hard to scan as a router

   If unsure, choose simple — it's easier to promote to hub-and-spoke later than to collapse an over-engineered structure.

3. **Determine skill name**:
   - MUST apply naming rules from references/skill-spec.md: lowercase, hyphens, max 64 chars
   - SHOULD prefer gerund form when natural (e.g., `managing-deploys`)
   - MUST confirm the name with the user -- suggest 1-3 name candidates derived from the requirements

4. **Create the skill directory**:
   - Create `<location>/<skill-name>/`
   - If the directory already exists, present options to the user: "Overwrite existing", "Pick a different name"

5. **Confirm the spec**:
   Before writing, confirm the full skill spec with the user. MUST summarize:
   - Skill name and directory path
   - Skill shape (simple or hub-and-spoke) with rationale if hub-and-spoke
   - Description (trigger keywords)
   - Operations to create (names and one-line summaries)
   - Reference files to create (hub-and-spoke only)
   - Frontmatter options (invocation mode, context, allowed-tools)

   Present options to the user: "Looks good", "Needs changes" (description: "I'll describe what to adjust"), "Start over" (description: "Re-gather requirements from scratch")
   - If "Needs changes", ask what to adjust, update, and re-confirm
   - If "Start over", return to step 1
   - MUST NOT proceed to writing until the user selects "Looks good"

6. **Write the skill**:
   Read the authoring specs (references/skill-spec.md, references/skill-template.md, references/shared-rules.md) and write the skill files inline:
   - Create `<skill_dir>/` via `mkdir -p`
   - **Simple shape**: Write a single SKILL.md with all operations inline, following the Simple Skill Template in references/skill-template.md
   - **Hub-and-spoke shape**: Write SKILL.md following the Hub-and-Spoke template: frontmatter (`name` matching directory, `description` per spec), Operations section (one H3 per operation with read gate), Combined Operations, References; write one `.md` per operation; write one `.md` per reference file with descriptive names
   - Validate against references/quality-checklist.md; fix any issues, up to 3 iterations

7. **Review and iterate**:
   Run the multi-perspective review loop per references/multi-perspective-review.md. Iterate until both agents pass or 4 cycles complete.

8. **Report results**:
   - MUST list all files created with a one-line description of each
   - MUST show the full `description` field so the user can verify trigger keywords
