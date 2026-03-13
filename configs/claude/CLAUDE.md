# User Preferences

## Conventions

RFC 2119 keywords (MUST, MUST NOT, SHALL, SHOULD, etc.) carry their defined meaning throughout all instructions. Treat them as binding constraints, not suggestions.

## Delegation

### Behavior

**The orchestrator decides. Subagents do.**

Pass the *problem*, not the *solution* — don't read code, diagnose issues, or prescribe implementations before delegating.

**Routing reads stay inline** — status checks, branch names, file existence, small lookups that inform the next decision, and reading file lists or directory structures to scope a delegation (determining which files to pass as context). Test: "am I gathering info to choose what to do next?" Diagnosing a behavioral problem in an agent or skill (e.g., "why did the skill produce identical output?") requires reading that agent/skill's implementation — this IS a routing read. Stop there: once you know which files need changing and why, delegate.

**Work gets delegated** — file analysis, diff review, artifact generation, multi-step execution. Test: "does this consume context I'll need later?"

**Stop before investigation becomes implementation** — reading a few files to identify what to delegate is a routing read. Reading files to figure out how to implement is doing the subagent's job.

**Heuristic**: if you've read more than 2-3 non-trivial files (excluding file lists, directory outputs, and small lookups) to understand a problem you plan to delegate, you've likely crossed the line. Delegate now with what you know and include the file paths as context for the subagent.

User interaction and state transitions stay in the orchestrator.

**Do NOT:**
- Read source files beyond what routing reads permit — reading to identify *what* to delegate is allowed (see routing reads above); reading to figure out *how* to implement is not
- Design implementations or prescribe code changes for subagents
- Re-read files a subagent already summarized
- Reduce a subagent to a transcriber by over-specifying the solution
- Delegate trivial routing reads that inform the orchestrator's next decision

### Routing

**Skills take precedence — this is a hard requirement.** Before dispatching any subagent, check whether an available skill covers the task. If a skill matches, MUST invoke it via the Skill tool — MUST NOT bypass the skill by routing directly to its subagents or doing the work inline. This applies whether or not the user used `/` syntax. The table below covers direct subagent dispatch only when no skill applies. Skills manage their own internal routing.

- Code (plan chunks) — `chunk-executor`
- Code (ad-hoc) — `code-writer`
- Commits — git skill (inline)
- Rules files — `rules-writer`
- Skill files — `skill-writer`
---

## Path Resolution

- Always resolve `tmp/` as `./tmp/` relative to the working directory, not as `/tmp/`.

## Repository Map

When the user references a repo by name (e.g., "check gadget", "look at the skill in dotfiles"), use this map to resolve the path.

**Convention:** All repos live under `~/Code/{personal,gadget,scratch}/<name>`. For repos not listed here, check those directories.

- `~/Code/personal/dotfiles` — macOS dotfiles, symlink-managed configs, **Claude Code skills**
- `~/Code/gadget/gadget` — Main Gadget monorepo (app platform)
- `~/Code/gadget/ggt` — Gadget CLI tool
- `~/Code/gadget/skipper` — Kubernetes operator for Gadget apps
- `~/Code/gadget/global-infrastructure` — Terraform/infra for Gadget cloud
