# Rewrite

Rewrites input text in the user's voice, adapting tone and structure to the target context.

1. **Identify input**: Use `$ARGUMENTS` as the source — treat it as a file path if it resolves to an existing file, as inline text otherwise. If empty, ask the user what text to rewrite via AskUserQuestion.

2. **Determine context**: Identify the target context from the user's request or the text itself. Default to "General technical writing" if ambiguous. Options:
   - PR description
   - Review comment (inline code review)
   - Discussion comment (PR/issue thread)
   - Slack message
   - General technical writing

3. **Rewrite**: Apply the style patterns below to rewrite the text:
   - MUST preserve all technical facts, names, links, and code references from the original
   - MUST match the detail level to the complexity of the content (terse for simple topics, thorough for complex ones)
   - MUST adapt structure to the target context (e.g., PR descriptions get problem-first structure; review comments stay terse and direct)
   - MUST NOT add information that wasn't in the original or that can't be inferred from it

4. **Verify**: Compare the rewrite against the original — confirm no facts were dropped or invented and that the tone matches the target context. Display a brief before/after comparison (key sentences from the original alongside the rewrite) so the user can validate the output.

5. **Present result**: Display the rewritten text. If the input came from a file, show the rewrite first, then ask whether to write it back. If the write fails (e.g., file not writable), display the rewritten text for the user to copy manually.

---

## Writing Style Reference

Codified writing style of scott-rc, derived from analysis of 1,249 authored PRs, 568 discussion comments, 1,794 inline review comments, and 727 PR review summaries from 2022–2025.

### Voice and Tone

- First person ("I", "we") is the default voice
- Conversational but professional — contractions are fine ("I'm", "don't", "we're", "it's")
- Direct sentences — lead with the point, not preamble
- Humble uncertainty when appropriate — "I think", "we suspect", "seems unlikely" for edge cases and debugging
- Self-aware about iteration — owns false starts and retries ("take 4 😅", strikethrough for abandoned approaches)

### Sentence Patterns

- Opens with "This" or "We" for action/change descriptions — "This adds...", "This fixes...", "We updated..."
- Short declarative sentences for simple facts
- Compound/complex sentences only when explaining causation — "We don't allow X because Y"
- Fragments are fine in comments for brevity — "Both worked ✅", "Just some updates"
- Questions framed as observations in reviews — "Should a ReadOnlySourceFile have a setData action on it?"

### Structure by Context

**PR Descriptions:**
- Simple changes (1-2 files, obvious fix) — 1-3 sentences max
- Medium changes (refactor, config, feature) — 1-2 paragraphs with context
- Complex changes (infrastructure, race conditions, new systems) — multi-section with "## Why" and "## How" headings
- Problem/context comes first, then the solution
- Links to related PRs, traces, or Slack threads at the end
- Before/after pattern for visual changes (screenshots) or behavioral changes (logs, metrics)
- /cc @person for tagging relevant people

**Examples:**

Terse — three sentences, no ceremony:
> This adds the ability to download and extract a specific cache version instead of always downloading the latest one.
>
> I made `-1` mean "give me the latest version", which is the same thing we do for `rebuild`.

Thorough — Why/How structure for infrastructure change (excerpt):
> This changes cached from using overlay to using LVM and thin snapshots.
>
> ## Why
>
> The overhead of using overlay to present the `dl_cache` to pods was too high, such that `dateilager-client rebuild`'s with the `dl_cache` using overlay were slower compared to `dateilager-client rebuild`'s without the `dl_cache` and no overlay.

Evidence-driven — observed symptom, then scenario breakdown:
> Our api-apps-shard-0 deployment had a moment with no active pods causing nginx to respond with 503s.
>
> The scenario that led to this happening was:
> - We had 4 api-apps-shard-0 pods
> - 3 of the 4 pods were shutdown because of a node scale down
> - The 1 left over pod couldn't handle all the traffic before the new pods spun up and started to fail its readiness probe

**Review Comments:**
- Single sentence or fragment for nits — "nit: should this be snake_case?"
- 2-5 sentences for logic suggestions, often with a code sample
- Questions over directives — "I think we should move this to..." rather than "Move this to..."
- Severity labels when needed — "nit:", leading question for suggestions
- Emoji reactions for approval — 🙌, 👍

**Examples:**

Owns decision with causality:
> I moved this "specialize on start" code here because `server.reloadOrchestrator` **has to be set** before we call `specialize`.

Question revealing oversight:
> Are we okay swallowing the cause if it's not an `Error`?

Evidence-driven feedback:
> I also reduced the number of times we emit these `worker-poll` events. I keep seeing these logs in Humio because of them.

**Discussion Comments:**
- Short and direct — typically 1-3 sentences
- Responds to quotes with context ("Correct." or "Pretty sure this is just...")
- Evidence-first when debugging — trace links, screenshots, log snippets before explanation
- Self-reporting on testing — "Just 🎩'd and this doesn't work..." or "Deployed this to sandbox-canary and it worked 🎉"

**Examples:**

Investigation with evidence trail:
> I built `4290a6c`, then built `7a84f99`, and every step was cached. IMO, this is evidence that our cache miss was due to "docker is missing the cache" rather than "docker thinks something actually changed".

Terse factual response:
> Yes exactly, it's the session ID from the cookie.
>
> `ggt` sends the same session ID on every request to Gadget. The session ID only changes if the user runs `ggt login` again and generates a new session.

**Slack Messages:**
- Casual and brief
- Often leads with the outcome or ask
- Uses thread context — doesn't repeat what was already said

**Examples:**

Investigation with before/after breakdown:
> Fission 1.20 is causing way more pods to specialize than necessary because of this change.
>
> Assuming no specialized pods for an app.
>
> Fission 1.19
> 1. 5 requests come in
> 2. Executor specializes 1 pod and queues 5 requests for it
> 3. 5 requests go to 1 specialized pod
>
> Fission 1.20
> 1. 5 requests come in
> 2. Executor specializes 5 pods and queues 1 request for each
> 3. 5 requests go to 5 different specialized pods

Terse status update during rollout:
> I've turned on ES for all apps that haven't used the search API in the last week
>
> I'm gonna leave this as is for the day, and if all goes well, I'll disable ts-vector updates for them all

Incident triage — outcome first, then evidence:
> This looks like a smoking gun imo. This query shows the timeline of emailmywebhooks (180332) hitting their sandbox max concurrency and causing the platform wide slowdown.
>
> And this query shows when wob (299610) request time spiked -- it lines up exactly when emailmywebhooks was hitting their max concurrency

### Technical Explanation Style

- Observable symptoms before root cause — describe what's happening, then why
- Evidence-driven — traces, metrics, screenshots, and log snippets as primary proof
- Precise numbers — "every 48 seconds", "2 minutes timeout", "11 minutes to complete" (not vague approximations)
- Explains "why" not just "what" — motivation for each change
- Code blocks for showing problems or solutions
- Numbered lists for multi-step sequences or multiple contributing causes
- details/summary tags for long output that's useful but secondary

### Emoji Usage

- Minimal and natural — not every message needs emoji
- ✅ — verification, confirmation
- 🎉 — something worked, shipped
- 🤷 — uncertainty, "not sure why but it works"
- 🤔 — questioning a convention or pattern
- 👍 — agreement, approval
- 🙌 — enthusiasm about a good change
- 😅 — self-aware humor about struggle
- 🚢 / :shipit: — ship it, approve
- 🔥 — something removed/deleted
- Skip emoji entirely when the message is purely technical

### Anti-Patterns

Things scott-rc does NOT do:

- Does NOT use corporate/marketing language — no "leverage", "synergize", "paradigm"
- Does NOT pad with filler — no "As mentioned above", "It's worth noting that", "In conclusion"
- Does NOT hedge excessively — says "I think" once, not "I think maybe perhaps"
- Does NOT use passive voice for things he did — "I fixed" not "it was fixed"
- Does NOT write walls of text for simple changes
- Does NOT omit context for complex changes
- Does NOT use ALL CAPS for emphasis (uses bold or italic instead)

### Verbal Tics and Recurring Phrases

- "This PR" — standard opener for PR descriptions
- "This adds/fixes/makes/removes" — change descriptions
- "We think" / "We suspect" — collaborative uncertainty
- "Just" — signals simplicity or admission ("Just some updates", "Just 🎩'd")
- "Worst case" — risk framing
- "imo" / "IMO" — opinion marker
- "/cc @person" — tagging for visibility
- "LGTM 👍" — standard approval
- Strikethrough for abandoned approaches — "~tried X, didn't work~"
