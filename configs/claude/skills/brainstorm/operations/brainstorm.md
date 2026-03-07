# Brainstorm

Collaborative ideation session — generate, riff on, and refine ideas with the user.

1. **Parse the topic**: Read the topic from `$ARGUMENTS`. If none provided, ask via AskUserQuestion. MUST NOT proceed without a clear topic.

2. **Read the room**: Gauge the topic's nature and calibrate your energy:
   - **Creative/open-ended** (naming, product ideas, content, side projects) — go wild. Use ideation techniques, propose absurd combinations, get excited about promising threads.
   - **Technical/constrained** (architecture decisions, API design, solving a specific problem) — still push past obvious answers, but ground ideas in feasibility. Weird analogies welcome; fantasy not.
   - **Strategic/business** (positioning, prioritization, process design) — challenge assumptions, invert the problem, explore contrarian takes. Balance boldness with pragmatism.

   Briefly state your read of the topic and what angles you'll explore. For ambiguous topics, confirm the frame via AskUserQuestion. Otherwise, dive in.

3. **Diverge — ideation rounds**: Generate and riff on ideas in freeform rounds. Each round SHOULD:
   - Throw out 3-5 ideas — bold titles with a sentence each
   - Use at least one ideation technique per round, chosen to fit the topic:
     - **Inversion** — "what if we did the opposite?"
     - **Analogy** — "what's the X of Y?" or borrow from an unrelated domain
     - **Mashup** — smash two prior ideas together and see what emerges
     - **Worst idea** — propose something deliberately terrible, then find the kernel of insight
     - **Constraint flip** — remove an assumed constraint, or add an artificial one
     - **Random stimulus** — pull in a concept from a completely unrelated field
   - Riff on the user's reactions from prior rounds — amplify what excited them, mutate what almost worked, abandon what fell flat
   - End with AskUserQuestion: what's hitting, what's not, where to push next

   Let the session breathe. Follow interesting threads even if they veer from the original angle. Circle back if needed. MUST adapt round count to complexity — 2 for simple topics, up to 5 for rich ones. End when energy converges or the user says so.

   SHOULD lead with the non-obvious. Safe, predictable ideas are filler — push past them.

4. **Converge — synthesize**: Distill the session into the 3-6 strongest ideas. For each:
   - Bold title
   - 1-2 sentence description
   - Why it stood out — what makes it interesting, novel, or actionable

   Present the synthesis and ask via AskUserQuestion: "Do these capture the best ideas? Anything to add, drop, or refine?" Update and re-confirm if the user adjusts.

5. **Deliver**: Keep the synthesis in context for follow-up. If the user asks to save or export, write to a file they specify or copy to clipboard.
