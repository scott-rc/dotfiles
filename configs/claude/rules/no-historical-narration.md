# No Historical Narration

In artifacts that live alongside code — comments, docstrings, plans, READMEs, documentation — describe the **current state**, not the diff or the journey to it. History rots when it lives next to the thing it claims to describe; it belongs in version control and PR descriptions.

## Rules

- **No before/after framing in persistent artifacts.** Scan for and rewrite: "previously", "used to", "no longer", "now does", "was changed to", "renamed from", "migrated from", "moved to", "extracted from", "switched to", "switched from", "was using", "we used to", "instead of (the old)", "rather than (the old)", "for backwards compatibility", "still supported", "as of v…", "in the new version", "replaced X with Y", "the old X", "originally we considered". (This list is the union of triggers Claude should self-scan for; it is kept in sync with the journey-pattern check in `configs/claude/skills/git/references/pr-writer-rules.md`.) If the sentence only makes sense when paired with knowledge of a prior state, delete or rephrase it.
- **No task / PR / issue / reviewer references in comments, docstrings, or doc prose.** Banned: `// added for the X flow`, `// fixes #123`, `// see PR #456`, `// per <name>'s feedback`. They rot; they belong in the PR description.
- **No tombstones for deleted code.** No `// formerly handled X`, no `// X was here`, no commented-out blocks, no `_unused_`-prefixed shims for symbols that no longer exist. If something is gone, it is gone. Live deprecation scaffolding is fine: a still-callable symbol marked `@deprecated`, a still-set feature flag, or a `TODO: remove once <concrete condition>` comment that names a real trigger. The ban is on remnants of code that is already gone, not on staged-removal infrastructure.
- **Plans describe the target, not the design path.** Write the destination. Don't write "we initially planned X but switched to Y" — write Y. The `plan` skill's Retrospective, `## Review findings`, and `## Phase N review-phase findings` sections are the only places plan files capture decision history.
- **Conversation prose: state results, not the journey.** The end-of-turn report describes the workspace's current state after this turn — "now" framing is fine there because the change just happened and the user asked for it. The ban applies only to persistent artifacts that ship alongside the code. Inside the turn, don't pad mid-task prose with "I first tried X, then switched to Y" unless the user asked for the journey.
- **Identifiers and user-visible strings count too.** Test names (`test_old_behavior`, `still_handles_legacy_input`), fixture paths (`before_migration.json`), and runtime error messages (`"foo is no longer supported"`) narrate history through identifiers that rot — and they're load-bearing, so they get cleaned up even less often than comments. Name them by current behavior: if you need to test legacy input handling, name the test after the input shape, not after its history.
- **Exceptions where historical framing IS the artifact's purpose:** PR descriptions, commit messages, CHANGELOG entries, migration guides, ADRs, plan Retrospectives. Even there, claims about prior state must match the diff — never invent before-states from session memory (the `git` skill enforces this for PRs).

## Example

Bad — in code:
```rust
// Previously returned Option<T>; now returns Result<T, E> for better errors.
// Renamed from `parse_loose` after the v2 refactor.
fn parse_strict(input: &str) -> Result<T, E> { ... }
```

Good:
```rust
fn parse_strict(input: &str) -> Result<T, E> { ... }
```

Bad — in a plan body:
> Originally we planned to extract a `Cache` trait, but after discussion decided to inline the LRU map into `Server`.

Good:
> Inline the LRU map into `Server`.

Bad — in a README:
> As of v2.0, the CLI no longer requires the `--legacy` flag.

Good:
> The CLI does not accept a `--legacy` flag.

When the historical fact IS the point — a breaking change a user is upgrading through — it belongs in the CHANGELOG or a migration guide, not in the README body. Move it; don't rewrite it into something semantically weaker. (Note: "no longer requires" and "does not accept" can mean different things; verify the rewrite preserves meaning, or relocate the sentence instead.)
