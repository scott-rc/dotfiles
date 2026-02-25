# Benchmark

Write a benchmark that captures the performance target, then write or optimize code to meet it — following the baseline-red-green-refactor cycle.

## Instructions

1. **Load coding guidelines**: Read [general-guidelines.md](general-guidelines.md), [testing-guidelines.md](testing-guidelines.md), [test-environment.md](test-environment.md), and any matching language-specific guidelines ([typescript-guidelines.md](typescript-guidelines.md), [go-guidelines.md](go-guidelines.md), [shell-guidelines.md](shell-guidelines.md)). Apply these throughout.

2. **Understand the performance goal**:
   Clarify what metric matters (throughput, latency, memory, binary size) and what the target is. If the user provides a specific target (e.g., "under 50ms", "10k ops/sec"), use it directly. If the goal is vague (e.g., "make it faster"), proceed to step 3.

3. **Establish baseline** (conditional):
   If the user says "make it faster" or "optimize this" without a specific target:
   - Spawn a Task subagent (type: Explore) to read the target code, identify hot paths and performance-relevant code structure, and return a concise analysis (function signatures, loop structures, allocation patterns, I/O calls)
   - Benchmark the current code to establish a baseline measurement
   - Present the baseline and propose 1-3 concrete targets via AskUserQuestion (e.g., "2x faster", "Under 50ms", a context-specific target)

4. **Plan benchmark cases**:
   Draft 1-3 benchmark cases covering the core performance scenario and any important variations (e.g., different input sizes, hot vs cold paths). Present the list to the user for confirmation before writing any code.

5. **Red — write a failing benchmark**:
   Write a benchmark with a threshold that the current code does not meet. If the current code already meets the target, report this to the user and stop — there is nothing to optimize.

6. **Verify failure**:
   Run the benchmark to confirm the current code does not meet the threshold. Record the result.

7. **Green — write or optimize implementation**:
   Write the simplest optimization that closes the gap between the current measurement and the target. Stop when the target is met.

8. **Verify pass**:
   Run the benchmark to confirm the target is met. MUST NOT weaken the benchmark threshold to pass — if the optimization is insufficient, iterate on the implementation.

9. **Refactor** (optional):
    SHOULD refactor if the optimization introduced unclear code, duplication, or maintainability concerns. Re-run the benchmark after refactoring to confirm the target is still met.

10. **Run existing tests**:
    MUST run the project's test suite (or relevant subset) to confirm the optimization did not break correctness. If tests fail, fix the implementation before proceeding.

11. **Report results**:
    MUST report to the user:
    - Baseline measurement (if established)
    - Final measurement
    - Improvement factor (e.g., "2.3x faster", "40% less memory")
    - Trade-offs introduced (e.g., increased memory usage, added complexity)
    - Test suite status
