---
name: reflect
description: Reflect candidly on the current session — by default, analyze how it could have gone better given different decisions or circumstances. Use when the user wants you to introspect, critique your own work, identify missed opportunities, analyze tool usage or communication, look back at what we did, or asks "what could have gone better".
argument-hint: "[topic]"
---

# Reflect

Look back over the current session and produce a candid, specific analysis. The point is to surface signal the user can act on — not to perform humility, not to reassure.

## Topic

- If the user passed a topic in the args, reflect on that.
- Otherwise, default to **counterfactuals**: what alternative paths existed at key decision points, and which ones likely would have been better.

Common topics the user might supply:
- *what went well* — name strong moments without inflating them
- *tool usage* — batching, dead ends, redundant reads, agent delegation
- *communication* — clarity, length, over-narration, premature conclusions
- *my prompting* — how the user's prompts shaped your work (honest, but kind — they asked)
- *time spent* — where the session burned cycles disproportionate to value delivered

## Process

1. **Scan the session.** Recall key decision points, dead ends, corrections, surprises, and any moment where you changed direction.
2. **Pick a few high-leverage observations** (typically 3–5). Quality over coverage.
3. **For each, name three things**: the moment (cite what was asked / what you did), the alternative path that existed, and what made the chosen path worse.
4. **Separate categories** so the user can tell what's actionable for whom:
   - **Agent decisions** — choices *you* made that you'd reconsider
   - **User decisions** — pivots or constraints set by the user that shaped the outcome
   - **Circumstances** — tool limits, missing context, ambiguous requirements, environmental friction
5. **End with the one or two patterns most worth carrying forward** — phrased as "next time I would…" not "X was suboptimal".

## Rules

- **Be specific.** Cite actual moments — what was asked, what you did, what happened. Vague reflection is worthless.
- **Be honest.** If you wasted time, say so. If a user choice constrained the outcome, say so plainly without blaming.
- **No sycophancy.** Don't pad reflection with reassurance. The user invoked this skill because they want signal.
- **No fabricated problems.** If the session genuinely went well and there's no meaningful counterfactual, say that directly. Inventing flaws to look humble is its own failure mode.
- **Don't catalog.** A few well-chosen observations beat a comprehensive list. If you find yourself writing a sixth bullet, ask whether it's pulling its weight.
- **Distinguish what you'd actually change from what was just situational.** "I'd batch those reads next time" is useful. "Tool calls were sequential" is just description.
- **Match length to substance.** A short session with one notable inflection point gets a short reflection. Don't stretch.

## Output format

Use this structure when reflecting on counterfactuals (the default). For other topics, adapt the section names but keep the "specific moment → alternative → why" pattern.

```md
## Reflection

### Agent decisions
- **<moment>** — <what I did>. Alternative: <what I could have done>. Why it would have been better: <reason>.

### User decisions
- **<moment>** — <constraint or pivot>. Tradeoff: <what it cost / enabled>.

### Circumstances
- **<factor>** — <how it shaped the outcome>.

### Carrying forward
- <one or two patterns, phrased as "next time I would…">
```

If the session went well with no meaningful counterfactual, skip the template and say so in one or two sentences.
