# Create Operation

Scaffold a new Claude Code skill interactively, producing a complete skill directory with SKILL.md, operation files, and reference files.

## Instructions

1. **Gather requirements**:
   Ask the user about:
   - What the skill does (purpose and domain)
   - What triggers it (user phrases that should invoke it)
   - What operations it needs (distinct tasks the skill should handle)
   - What shared knowledge operations need (patterns, guidelines, templates)
   - Any runtime dependencies (CLIs, APIs, services)
   - Where to create it (default: `~/.claude/skills/` or the project's skill directory)

2. **Determine skill name**:
   - Apply naming rules from [spec.md](spec.md): lowercase, hyphens, max 64 chars
   - Prefer gerund form when natural (e.g., `managing-deploys`)
   - Confirm the name with the user before proceeding

3. **Create the skill directory**:
   - Create `<location>/<skill-name>/`
   - If the directory already exists, ask the user whether to overwrite or pick a different name

4. **Write SKILL.md**:
   - Use the SKILL.md template from [skill-template.md](skill-template.md)
   - Write the frontmatter: `name` matching directory name, `description` following [spec.md](spec.md) description rules, optional `compatibility` if dependencies exist
   - Write the Operations section with one H3 per operation, each with a one-line summary and file link
   - Write Combined Operations if multiple operations can be chained (map user phrases to operation sequences)
   - Write References section linking any shared reference files

5. **Write operation files**:
   - One `.md` file per operation, named after the operation (e.g., `deploy.md`)
   - Use the operation file template from [skill-template.md](skill-template.md)
   - Each file has: H1 heading matching the SKILL.md operation name, one-line summary, numbered steps with bold step names
   - Steps should be specific and actionable -- tell the agent exactly what to do, not vague guidance
   - Include decision points for conditional logic ("If X, do Y. Otherwise, do Z.")
   - Include error handling for likely failure modes
   - End each operation with a step that reports results to the user

6. **Write reference files**:
   - One `.md` file per shared knowledge area (guidelines, patterns, templates, checklists)
   - Name files descriptively: `<topic>-<type>.md` (e.g., `commit-guidelines.md`, `deploy-patterns.md`)
   - Reference files provide information, not step-by-step instructions
   - Keep each reference file focused on one topic

7. **Validate the skill**:
   - Run through [quality-checklist.md](quality-checklist.md) mentally
   - Check: frontmatter is valid, all linked files exist, no orphan files, names follow rules, no redundancy between files
   - Fix any issues found before reporting to the user

8. **Report results**:
   - List all files created with a one-line description of each
   - Show the full `description` field so the user can verify trigger keywords
   - Suggest next steps: test the skill by invoking it, run the review operation to catch issues, iterate on operation details based on real usage
