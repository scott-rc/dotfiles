# Interview

Conduct a structured, multi-round interview with the user on any topic, then produce a summary or enrich session context.

## Instructions

1. **Parse the topic**:
   Read the topic from `$ARGUMENTS`. If no arguments provided, ask the user what topic to explore.

   MUST NOT proceed until a clear topic is established.

2. **Set the frame**:
   Before asking questions, briefly state:
   - What you understand the topic to be
   - What kind of questions you plan to ask (scope, depth, angle)
   - Roughly how many rounds you expect (suggest 3-5, adjust based on complexity)

   For complex or ambiguous topics, present the frame and ask the user to confirm or redirect before proceeding. For simple, clear topics, proceed directly to questioning.

3. **Conduct the interview**:
   Ask questions in rounds. Each round SHOULD:
   - Ask 2-4 related questions grouped by theme
   - Use the AskUserQuestion tool to batch questions into a single interaction
   - Build on prior answers -- do not repeat or re-ask what was already covered

   Question strategy:
   - **Round 1**: Broad strokes -- goals, motivation, constraints, who/what is involved
   - **Subsequent rounds**: Drill into areas the user's answers reveal as complex, ambiguous, or important
   - **Final round**: Edge cases, risks, open questions, anything the user wants to add

   MUST adapt the number of rounds to the topic's complexity. Simple topics need 2-3 rounds. Complex topics MAY need 5+. End the interview when answers start converging or the user signals they're done.

   SHOULD prioritize depth over breadth -- it is better to thoroughly explore 3 important areas than to superficially cover 10.

4. **Summarize understanding**:
   Write a concise summary of everything learned (aim for 3-6 top-level headings, under 500 words unless complexity demands more). Structure it with clear headings that match the topic's natural shape (not the question order).

   Announce "The interview is complete." then present the summary to the user: "Does this capture everything accurately? Anything to add or correct?"

   If the user corrects anything, update and re-confirm.

5. **Deliver results**:
   Default to keeping the summary in context for follow-up work. If the user asks to save or share, offer: copy to clipboard (`pbcopy`), save to a file, or feed into another task (ask what task, then invoke the relevant skill with the summary as context).
