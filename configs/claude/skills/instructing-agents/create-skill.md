# Create Skill

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
   - MUST apply naming rules from the Skill Specification section of [spec.md](spec.md): lowercase, hyphens, max 64 chars
   - SHOULD prefer gerund form when natural (e.g., `managing-deploys`)
   - MUST confirm the name with the user before proceeding

3. **Create the skill directory**:
   - Create `<location>/<skill-name>/`
   - If the directory already exists, ask the user whether to overwrite or pick a different name

4. **Write SKILL.md**:
   - MUST use the SKILL.md template from [skill-template.md](skill-template.md)
   - MUST write the frontmatter: `name` matching directory name, `description` following the Skill Specification section of [spec.md](spec.md)
   - MUST write the Operations section with one H3 per operation, each with a one-line summary and file link
   - SHOULD write Combined Operations if multiple operations can be chained (map user phrases to operation sequences)
   - SHOULD write References section linking any shared reference files

5. **Write operation files**:
   - MUST create one `.md` file per operation, named after the operation (e.g., `deploy.md`)
   - MUST use the operation file template from [skill-template.md](skill-template.md)
   - MUST include: H1 heading matching the SKILL.md operation name, one-line summary, numbered steps with bold step names
   - MUST include decision points for conditional logic ("If X, do Y. Otherwise, do Z.")
   - MUST end each operation with a step that reports results to the user
   - Steps MUST be specific and actionable -- tell the agent exactly what to do, not vague guidance
   - SHOULD include error handling for likely failure modes
   - SHOULD choose the right degree of freedom for each step (see Skill Content Rules in [spec.md](spec.md))
   - SHOULD apply content patterns from [skill-template.md](skill-template.md) where they fit: feedback loops for quality-critical steps, checklists for multi-step tracking, examples for ambiguous output

6. **Write reference files**:
   - SHOULD create one `.md` file per shared knowledge area (guidelines, patterns, templates, checklists)
   - MUST name files descriptively: `<topic>-<type>.md` (e.g., `commit-guidelines.md`, `deploy-patterns.md`)
   - MUST provide information, not step-by-step instructions
   - SHOULD keep each reference file focused on one topic

7. **Validate the skill**:
   - MUST run through [quality-checklist.md](quality-checklist.md)
   - MUST check: frontmatter is valid, all linked files exist, no orphan files, names follow rules, no redundancy between files
   - MUST fix any issues found before reporting to the user

8. **Report and evaluate**:
   - MUST list all files created with a one-line description of each
   - MUST show the full `description` field so the user can verify trigger keywords
   - SHOULD build evaluations before finalizing: create at least one test scenario per operation as a JSON object with `input` (user phrase), `expected` (correct behavior), and `criteria` (how to judge pass/fail)
   - SHOULD run each evaluation: invoke the skill with the test input, compare output against criteria, fix the skill if it fails
   - SHOULD run the review operation on the new skill to catch structural issues
