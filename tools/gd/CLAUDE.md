# tools/gd

@README.md

---

Use bare unwrap where safe. See `.claude/rules/tools.md` for build and shared error-handling requirements.

---

Diff color constants live at the top of `style.rs`. Use `style::` prefix everywhere, not inline ANSI codes.

---

Fix clippy lints idiomatically -- narrow `pub` to `pub(crate)` for crate-internal items, refactor to reduce argument counts, use `let...else` and `is_none_or`. MUST NOT suppress with `#[allow]` unless the lint is genuinely inapplicable.

---

When optimizing performance, MUST run `cargo bench` before and after changes. Use `cargo bench --bench bench -- --save-baseline before` to save a baseline, then `cargo bench --bench bench -- --baseline before` to compare. Use `samply record` on the release binary for flamegraph profiling.

For end-to-end startup benchmarking, use `--replay q` (NOT `--no-pager` -- it disables color when piped, hiding the real bottleneck). Use `GD_DEBUG=1` to get phase-level timing: `GD_DEBUG=1 gd --replay q 2>timing.txt`.

---

## Web UI Profiling (for Agents)

To profile the gd web UI performance:

### Setup

1. Reset test fixture: `./e2e/fixtures/setup.sh`
2. Start server: `cd e2e/fixtures/test-repo && gd --web --no-open`
3. Parse URL from stderr (e.g., `http://127.0.0.1:3845`)

### Using Chrome DevTools MCP Tools

```
# Open page
mcp__chrome-devtools__new_page { url: "http://127.0.0.1:3845" }

# Get app-level metrics
mcp__chrome-devtools__evaluate_script { expression: "window.__gdPerf.getMetrics()" }

# Capture performance trace
mcp__chrome-devtools__performance_start_trace {}
# ... interact with page (press keys, navigate) ...
mcp__chrome-devtools__performance_stop_trace {}
mcp__chrome-devtools__performance_analyze_insight { insight: "LCPBreakdown" }

# Memory analysis
mcp__chrome-devtools__take_memory_snapshot {}
```

### Key Metrics & Targets

| Metric | Target | Access |
|--------|--------|--------|
| Initial load | <100ms | `__gdPerf.getMetrics().initialLoad.totalMs` |
| Render time | <20ms | `__gdPerf.getMetrics().render.lastMs` |
| Navigation latency | <10ms | `__gdPerf.getMetrics().navigation` |
| DOM nodes | <2000 | `__gdPerf.getMetrics().dom.nodeCount` |

### Test Scenarios

- Initial load: open page, measure `firstRenderTime`
- Navigation: 50x j/k keypresses, check `navigationTimes` avg/p95
- Hunk jump: 20x ] keypresses
- Tree toggle: 10x l keypresses
- Search: open with /, type query, n/N navigation
