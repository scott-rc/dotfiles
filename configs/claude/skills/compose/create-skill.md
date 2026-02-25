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

4. **Write SKILL.md**:
   - MUST use the SKILL.md template from [skill-template.md](skill-template.md)
   - MUST write the frontmatter: `name` matching directory name, `description` following [skill-spec.md](skill-spec.md)
   - SHOULD include optional frontmatter fields when applicable: `disable-model-invocation`, `user-invocable`, `allowed-tools`, `model`, `context`, `agent`, `argument-hint`
   - MUST distinguish task vs reference content and set invocation control accordingly (see Skill Content Rules in [skill-spec.md](skill-spec.md))
   - MUST read [shared-rules.md](shared-rules.md) and apply Content Rules to all written content
   - MUST write the Operations section with one H3 per operation, each with a one-line summary and file link
   - SHOULD write Combined Operations if multiple operations can be chained (map user phrases to operation sequences)
   - SHOULD write References section linking any shared reference files

5. **Design delegation boundaries**:
   For each operation, identify what the orchestrator does (user interaction) vs what subagents do (everything else). MUST apply the subagent delegation and named agent patterns from [content-patterns.md](content-patterns.md).

6. **Write operation files**:
   - MUST create one `.md` file per operation, named after the operation (e.g., `deploy.md`)
   - MUST use the operation file template from [skill-template.md](skill-template.md)
   - MUST include: H1 heading matching the SKILL.md operation name, one-line summary, numbered steps with bold step names
   - MUST include decision points for conditional logic ("If X, do Y. Otherwise, do Z.")
   - MUST end each operation with a step that reports results to the user
   - Steps MUST be specific and actionable -- tell Claude exactly what to do, not vague guidance
   - MUST design steps around the delegation boundaries from step 5 -- orchestrator steps handle user interaction, subagent steps handle the work
   - SHOULD include error handling for likely failure modes
   - SHOULD choose the right degree of freedom for each step (see Skill Content Rules in [skill-spec.md](skill-spec.md))
   - SHOULD apply content patterns from [content-patterns.md](content-patterns.md) where they fit: feedback loops for quality-critical steps, checklists for multi-step tracking, examples for ambiguous output

7. **Write reference files**:
   - SHOULD create one `.md` file per shared knowledge area (guidelines, patterns, templates, checklists)
   - MUST name files descriptively: `<topic>-<type>.md` (e.g., `commit-guidelines.md`, `deploy-patterns.md`)
   - MUST provide information, not step-by-step instructions
   - SHOULD keep each reference file focused on one topic

8. **Validate and evaluate**:
   Spawn a Task subagent (type: skill-reviewer) with the new skill directory path. The skill-reviewer agent reads all files, validates structure, evaluates content quality, and checks for anti-patterns. It returns findings grouped by severity with per-file token counts.

   SHOULD write at least one test scenario per operation: a user phrase, the expected behavior, and how to judge pass/fail. Format as a markdown list, not JSON.
   SHOULD run each evaluation: invoke the skill with the test input, compare output against criteria, fix the skill if it fails.
   MUST fix any issues found before reporting to the user.

9. **Report results**:
   - MUST list all files created with a one-line description of each
   - MUST show the full `description` field so the user can verify trigger keywords
