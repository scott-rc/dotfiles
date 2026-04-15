/** Performance metrics exposed on window.__gdPerf for profiling. */

interface PerfMetrics {
  initialLoad: { totalMs: number };
  render: { lastMs: number; count: number };
  navigation: { times: number[] };
  dom: { nodeCount: number };
}

const startTime = performance.now();
let firstRenderDone = false;
let renderCount = 0;
let lastRenderMs = 0;
const navigationTimes: number[] = [];

function getMetrics(): PerfMetrics {
  return {
    initialLoad: {
      totalMs: firstRenderDone ? lastRenderMs : performance.now() - startTime,
    },
    render: { lastMs: lastRenderMs, count: renderCount },
    navigation: { times: [...navigationTimes] },
    dom: { nodeCount: document.querySelectorAll("*").length },
  };
}

export function markRender(durationMs: number) {
  renderCount++;
  lastRenderMs = durationMs;
  if (!firstRenderDone) {
    firstRenderDone = true;
    lastRenderMs = performance.now() - startTime;
  }
}

export function markNavigation(durationMs: number) {
  navigationTimes.push(durationMs);
  // Keep last 100 entries
  if (navigationTimes.length > 100) {
    navigationTimes.shift();
  }
}

// Expose on window
(window as unknown as Record<string, unknown>).__gdPerf = {
  getMetrics,
  markRender,
  markNavigation,
};
