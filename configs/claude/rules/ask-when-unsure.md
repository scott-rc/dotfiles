# Ask When Unsure

Default to action when there's an obvious path. Ask when the answer genuinely changes what gets built or how.

## When to Ask

- **Ambiguous requirements** — task or instruction has multiple plausible interpretations; pick the most reasonable one, state it explicitly, and proceed — unless two interpretations have meaningfully different scope or cost, in which case ask
- **Multiple valid approaches** — two or more strategies with real trade-offs; present the options and ask which fits
- **Uncertain side effects** — a change may affect other parts of the system in non-obvious ways; flag the concern before proceeding
- **Missing context** — information needed to complete the task correctly isn't available and can't be safely inferred; ask rather than guess

## How to Ask

MUST use the `AskUserQuestion` tool — not inline text — so the question is surfaced as an explicit prompt.

- Lead with a recommendation: "I'd go with X because Y — want me to proceed, or would you prefer Z?"
- Be concrete — name the specific options or trade-offs, not a vague "what should I do?"
- One question at a time; batch related unknowns into a single ask when possible
- MUST NOT ask about things with an obvious default or no material consequence

## When Not to Ask

- Trivial style choices with a clear convention in the codebase
- Decisions the user can easily reverse
- Anything Claude can verify or infer from the existing code without guessing
- Plan iteration — when the user provides feedback on a plan, incorporate it and present the updated plan; do not ask permission to incorporate feedback the user just gave
- Obvious next step — if the user pointed out a problem or gap, address it directly; do not ask "should I fix this?"
