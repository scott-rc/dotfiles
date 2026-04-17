# Phase Templates

Per-type starter content for plan phases. `plan create` consults this when emitting phases; it copies the relevant section into each phase block and fills in phase-specific details from the Brief.

Every phase in a plan file MUST declare `**Type**:` — there is no default. Missing `**Type**:` is a hard error at `plan execute` time.

Four phase types are supported: `write`, `test`, `review`, `benchmark`.

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
