# Create Skill

Scaffold a new Claude Code skill interactively, producing a complete skill directory with SKILL.md, operation files, and reference files.

## Instructions

1. **Gather requirements**:
   Ask the user via AskUserQuestion about:
   - What the skill does (purpose and domain)
   - What triggers it (user phrases that should invoke it)
   - What operations it needs (distinct tasks the skill should handle)
   - What shared knowledge operations need (patterns, guidelines, templates)
   - Any runtime dependencies (CLIs, APIs, services)
   - Where to create it -- present applicable locations as AskUserQuestion options (e.g., `~/.claude/skills/`, project `.claude/skills/`)
   - Who invokes the skill -- present as AskUserQuestion options: "User only" (description: `disable-model-invocation: true` for side-effect workflows), "Claude only" (description: `user-invocable: false` for background knowledge), "Both" (description: default)
   - Should it run in a subagent (`context: fork`) or inline? Present as AskUserQuestion options.
   - Any tool restrictions needed (`allowed-tools`)?

2. **Determine skill name**:
   - MUST read [skill-spec.md](skill-spec.md) before proceeding
   - MUST apply naming rules from [skill-spec.md](skill-spec.md): lowercase, hyphens, max 64 chars
   - SHOULD prefer gerund form when natural (e.g., `managing-deploys`)
   - MUST confirm the name with the user -- suggest 1-3 name candidates derived from the requirements via AskUserQuestion

3. **Create the skill directory**:
   - Create `<location>/<skill-name>/`
   - If the directory already exists, present options via AskUserQuestion: "Overwrite existing", "Pick a different name"

4. **Write the skill**:
   Spawn a Task subagent (type: skill-writer) in create mode. Pass:
   - `mode`: create
   - `skill_dir`: the absolute path from step 3
   - `spec`: the gathered requirements from steps 1-2 (name, description, operations, references, frontmatter options, delegation boundaries)

   The skill-writer reads authoring specs, writes all files (SKILL.md, operations, references), validates against the quality checklist, and self-corrects up to 3 iterations. It returns the list of files created, validation status, and per-file token counts.

   MUST fix any blocking issues the skill-writer reports before proceeding.

5. **Review and iterate**:
   Run the multi-perspective review loop per [multi-perspective-review.md](multi-perspective-review.md). Iterate until all 3 agents pass or 4 cycles complete.

6. **Report results**:
   - MUST list all files created with a one-line description of each
   - MUST show the full `description` field so the user can verify trigger keywords
