# Create Skill

Scaffold a new Claude Code skill interactively, producing a complete skill directory with SKILL.md, operation files, and reference files.

## Instructions

1. **Gather requirements**:
   Follow the interview pattern from references/content-patterns.md. Ask the user via AskUserQuestion about:
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
   Spawn a Task subagent (type: skill-writer) in create mode. Pass:
   - `mode`: create
   - `skill_dir`: the absolute path from step 3
   - `spec`: the confirmed requirements from step 4 (name, description, operations, references, frontmatter options, delegation boundaries)

   Expect back: list of files created, validation status, and per-file token counts.

   MUST fix any blocking issues the skill-writer reports before proceeding.

6. **Verify Alloy spec** (if the skill has a `specs/` directory containing `.als` files):
   Run `alloy exec -f -o /tmp/alloy-output <spec-path>` for each `.als` file. If any check returns SAT (counterexample found), the skill violates a behavioral invariant. Read the Alloy output to identify which assertion failed, map it back to the skill files, and fix before proceeding. Re-run until all checks return UNSAT.

7. **Review and iterate**:
   Run the multi-perspective review loop per references/multi-perspective-review.md. Iterate until all 3 agents pass or 4 cycles complete.

8. **Report results**:
   - MUST list all files created with a one-line description of each
   - MUST show the full `description` field so the user can verify trigger keywords
