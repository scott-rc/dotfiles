# Phase Templates

Per-type starter content for plan phases. `plan create` consults this when emitting phases; it copies the relevant section into each phase block and fills in phase-specific details from the Brief.

Every phase in a plan file MUST declare `**Type**:` — there is no default. Missing `**Type**:` is a hard error at `plan execute` time.

Five phase types are supported: `write`, `test`, `review`, `benchmark`, `audit`.

---

## Type: write

Most phases are `write`. The phase changes runtime behavior (new feature, bug fix, refactor) or changes config/glue. `plan execute` loads `code/operations/write.md` and follows it inline. Write mode internally picks TDD (for behavior changes / bug fixes) or Apply (for refactoring under existing coverage, config, shell glue, one-liners) per the `code` skill's own rules.

### Phase title convention

Noun-phrase describing the slice: "Introduce SearchState type," "Migrate match computation," "Delete old fields." Not imperative ("Add X" reads fine too). Keep under ~60 chars.

### Starter acceptance criteria

```
- [ ] <user-visible behavior described in the Brief is now observable>
- [ ] Existing tests pass (`cargo test` / `npm test` / etc.)
- [ ] New tests written for new behavior (when TDD mode applies)
- [ ] Build passes without new warnings
- [ ] Linter / formatter clean
```

Add phase-specific behavior criteria derived from the Brief's Proposed Interface or user stories.

### Customization guidance

- For bug-fix sub-phases: add "- [ ] Regression test reproduces the original bug and now passes."
- For refactor slices under existing coverage: swap "New tests written" for "Existing tests continue to pass unchanged."
- For config/glue changes: drop test criteria; keep "Build passes" and add "- [ ] The config change produces the documented effect (manually verified)."

---

## Type: test

The phase adds test coverage without changing runtime behavior. `plan execute` loads `code/operations/test.md` and follows it inline. Test mode supports two sub-modes: Coverage (characterization tests for untested code) and Mutate (mutation testing to find and kill survivors).

Use this phase type when the work is purely about tests — backfilling coverage on legacy code, hardening an existing module with mutation testing, or adding boundary tests after a refactor.

### Phase title convention

"Backfill tests for <module>," "Harden <module> with mutation testing," "Add boundary tests at <seam>."

### Starter acceptance criteria

```
- [ ] <coverage target> met (e.g. 90%+ lines on `<module>` per `./coverage.sh`)
- [ ] Tests assert observable behavior through public interfaces, not internal state
- [ ] No test depends on internals that would break under refactor
- [ ] All tests pass
```

For Mutate mode, add:
```
- [ ] Mutation testing run; survivors triaged
- [ ] Each killed mutant is accompanied by a test case that catches it
- [ ] Documented surviving mutants (equivalent / not worth catching) with rationale
```

### Customization guidance

- Always cite the specific coverage / mutation target from the Brief's Testing Strategy.
- If the Brief specifies particular behaviors to characterize, list them as explicit criteria.

---

## Type: review

The terminal phase of most plans. Combines static verification (dispatches to `code review`) and behavioral verification (orchestrator drives the running artifact). Runs as an evaluate-fix loop per `code/operations/review.md`. Criteria are derived from the Brief's `### Review Criteria` section, which splits into `**Code**:` and `**Behavior**:` sub-lists.

### Phase title convention

Just "Review." Or "Review and verify" if the Brief has a meaningful `**Behavior**:` section worth naming.

### Starter structure

```
## Phase N: Review

**Type**: review

### What to verify

Confirm the work matches the Brief. Runs `code review` for static checks plus interactive behavioral verification for runtime criteria.

### Acceptance criteria

**Code**:
- [ ] <each **Code**: bullet from the Brief's Review Criteria becomes a checkbox>

**Behavior**:
- [ ] <each **Behavior**: bullet from the Brief's Review Criteria becomes a checkbox>
```

If the Brief has no Review Criteria section, emit these defaults:

```
**Code**:
- [ ] Implementation matches the Brief's Proposed Interface (or user stories)
- [ ] No new lint warnings introduced
- [ ] Test coverage is appropriate for the change

**Behavior**:
- [ ] User-visible behavior described in the Brief works end-to-end
```

### Customization guidance

- Copy Review Criteria verbatim from the Brief; do not rephrase. The Brief is the spec.
- If the Brief's `**Behavior**:` list is empty (pure refactor, no user-visible change), the behavioral verification step still runs a smoke check — confirm the build still boots and existing behavior is preserved.

### Non-convergence

If the loop does not converge (no progress iteration or recurring finding), `plan execute` halts WITHOUT committing. See `plan/operations/execute.md` for the full behavior and user-prompt flow.

---

## Type: benchmark

The phase establishes a performance target and confirms it's met. `plan execute` loads `code/operations/benchmark.md` and follows it inline. Benchmark mode writes or updates a benchmark that captures the target, then writes or optimizes code until the benchmark passes.

### Phase title convention

"Benchmark <scenario>," "Meet <target> on <scenario>," "Optimize <module> startup."

### Starter acceptance criteria

```
- [ ] Benchmark exists and reproducibly measures the target scenario
- [ ] Baseline captured (`cargo bench --save-baseline before` or equivalent)
- [ ] Target met or exceeded; comparison vs baseline reported
- [ ] No regression in related benchmarks
```

### Customization guidance

- Pull the quantitative target directly from the Brief (e.g. "< 200ms startup on a 65-file diff").
- If the target is a *ratio* rather than an absolute (e.g. "2x faster than current"), state the measured baseline in the acceptance criteria so the target is pinned to a concrete number.

---

## Type: audit

> **Status: STUB.** This Type exists because "sweep the system for issues, surface findings, triage with the user, apply approved fixes" is a distinct workflow from TDD-shaped `write`, test-coverage-shaped `test`, or terminal `review`. The protocol below is the minimum viable spec. It SHOULD be refined and filled out during the first real use, and the learnings committed back into this document.

An audit phase scans across a defined surface (files, modules, dependencies, configuration, docs), surfaces findings per a defined category set, lets the user triage them (fix now / defer / out of scope), and applies approved fixes in a single commit. The surface and category set are always defined in the phase's `### What to build` section — the phase itself is what makes an audit phase specific. Acceptance criteria are usually defined by *coverage* (was the whole surface audited across every category?) rather than by a fixed finding list (which is unknown until the audit runs).

Audit is orchestrator-owned in `plan execute` — there is no `Skill(code, audit)` dispatch. The orchestrator follows the guidance here plus the phase's `### What to build`.

### Phase title convention

"Audit <surface>," "Audit and reconcile <surface>." Concrete about the surface: "Audit repo documentation," "Audit cross-service dependency declarations," "Audit security-sensitive endpoints."

### Process (generic)

1. Enumerate the audit surface per the phase's What-to-build.
2. For each item in the surface, check every category the phase specifies (drift, consistency, conflicts, design issues, etc. — phase-specific).
3. Consolidate findings. Dedupe across files.
4. Present findings to the user, grouped by category. Get triage per finding: fix now / defer (with rationale) / out of scope (with rationale).
5. Apply approved fixes.
6. Commit once per phase per the usual rule.

### Starter acceptance criteria

```
- [ ] Every item in the audit surface (as enumerated in What to build) was checked across every category defined by the phase
- [ ] Findings were presented to the user and triaged; approved fixes applied; deferred / out-of-scope items captured with rationale
- [ ] Existing tests pass (if the fixes touched anything under test coverage)
- [ ] Build / lint passes (if the fixes touched buildable code)
```

### Customization guidance

- Enumerate the audit surface concretely in `### What to build` — files, globs, modules, endpoints. An audit whose surface is ambiguous will produce ambiguous acceptance criteria.
- Enumerate the category set concretely too — "drift," "consistency," "conflicts," "design issues," etc., each with a one-sentence definition of what that category looks like in this phase's context.
- Expect the triage step to be interactive; don't pre-commit to a fix set.
- When the audit surfaces items clearly out of scope for the phase, propose them to `plan review` as candidates for New plans (not Fixup phases).

### Meta-note

First real use (as of this writing): Phase 1 of `tmp/dotfiles-drift/plan.md`. Whatever protocol that execution settles into — how findings are formatted, how triage is presented, how the commit is structured — should be captured back into the Process section above as concrete guidance, replacing the generic sketch.
