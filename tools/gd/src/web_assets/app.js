// gd web — client state machine & rendering
"use strict";

// ---------------------------------------------------------------------------
// State
// ---------------------------------------------------------------------------
const state = {
  files: [],
  tree: [],
  // Flattened line list for the diff pane
  flatLines: [],    // { type: 'file-header'|'hunk-sep'|'line', fileIdx, hunkIdx, lineIdx, data }
  fileStarts: [],   // flatLines index where each file begins
  hunkStarts: [],   // flatLines index where each hunk begins
  changeGroupStarts: [], // flatLines index where each change group begins (like TUI)
  cursorLine: 0,
  viewScope: 'all', // 'all' | 'single'
  singleFileIdx: 0,
  treeFocused: false,
  treeVisible: true,
  treeCursor: 0,
  treeWidth: 260,   // Tree panel width in pixels
  resizing: false,  // Tree resize drag state
  pendingTreeKey: null,
  searchQuery: '',
  searchMatches: [],
  searchCurrentIdx: -1,
  searchActive: false,
  helpVisible: false,
  collapsedDirs: new Set(),
  expandedDirs: new Set(),
  fullContext: false,
  visualAnchor: null, // null or line index for visual selection
  theme: 'system',  // 'light' | 'dark' | 'system'
  // Virtual rendering state
  renderedRange: { start: 0, end: 0 },
  lineOffsets: [],    // Pre-computed cumulative offsets for each line
  totalHeight: 0,     // Total scrollable height
  scrollScheduled: false,
};

// Expose state for e2e tests (access via window.__gdState in Playwright)
if (typeof window !== 'undefined') {
  window.__gdState = state;
}

// ---------------------------------------------------------------------------
// Performance instrumentation for agent profiling
// ---------------------------------------------------------------------------
const __gdPerf = {
  ready: false,
  wsConnectTime: null,
  firstRenderTime: null,
  lastRenderStart: null,
  lastRenderDuration: null,
  navigationTimes: [],

  markWsConnect() { this.wsConnectTime = performance.now(); },
  markFirstRender() {
    this.firstRenderTime = performance.now();
    this.ready = true;
  },
  startRender() { this.lastRenderStart = performance.now(); },
  endRender() {
    if (this.lastRenderStart) {
      this.lastRenderDuration = performance.now() - this.lastRenderStart;
    }
  },
  recordNavigation(key, duration) {
    this.navigationTimes.push({ key, duration, ts: Date.now() });
    if (this.navigationTimes.length > 100) this.navigationTimes.shift();
  },

  getMetrics() {
    return {
      initialLoad: {
        wsConnectMs: this.wsConnectTime,
        firstRenderMs: this.firstRenderTime,
        totalMs: this.firstRenderTime
      },
      render: { lastMs: this.lastRenderDuration },
      navigation: this.navigationTimes.slice(-20),
      dom: {
        nodeCount: document.querySelectorAll('*').length,
        diffLineCount: document.querySelectorAll('.diff-line').length
      },
      memory: performance.memory ? {
        usedMB: performance.memory.usedJSHeapSize / 1024 / 1024,
        totalMB: performance.memory.totalJSHeapSize / 1024 / 1024
      } : null
    };
  }
};
window.__gdPerf = __gdPerf;

// ---------------------------------------------------------------------------
// Virtual rendering constants
// ---------------------------------------------------------------------------
const LINE_HEIGHT = 20;       // .diff-line min-height
const HEADER_HEIGHT = 35;     // .file-header approximate height
const HUNK_SEP_HEIGHT = 20;   // .hunk-sep height
const BUFFER_LINES = 30;      // Extra lines above/below viewport to render

// DOM refs
const treeEl = document.getElementById('tree');
const diffPane = document.getElementById('diff-pane');
const statusLeft = document.getElementById('status-left');
const statusRight = document.getElementById('status-right');
const searchBar = document.getElementById('search-bar');
const searchInput = document.getElementById('search-input');
const searchCount = document.getElementById('search-count');
const helpOverlay = document.getElementById('help-overlay');
const resizeHandle = document.getElementById('resize-handle');
const themeToggle = document.getElementById('theme-toggle');

// ---------------------------------------------------------------------------
// WebSocket
// ---------------------------------------------------------------------------
let ws = null;

function connect() {
  const proto = location.protocol === 'https:' ? 'wss:' : 'ws:';
  ws = new WebSocket(`${proto}//${location.host}/ws`);

  ws.onopen = () => { __gdPerf.markWsConnect(); };

  ws.onmessage = (ev) => {
    const msg = JSON.parse(ev.data);
    if (msg.type === 'DiffData') {
      state.files = msg.files;
      state.tree = msg.tree;
      // Pre-populate expandedDirs so all directories start expanded
      state.expandedDirs = initExpandedDirs(msg.tree);
      flattenLines();
      // Focus first change group on initial load (like TUI)
      focusFirstChangeGroup();
      renderAll();
      if (!__gdPerf.ready) __gdPerf.markFirstRender();
    }
  };

  ws.onclose = () => {
    ws = null;
    setTimeout(connect, 2000);
  };
}

function sendMessage(obj) {
  if (ws && ws.readyState === WebSocket.OPEN) {
    ws.send(JSON.stringify(obj));
  }
}

function toggleFullContext() {
  state.fullContext = !state.fullContext;
  sendMessage({ type: 'SetFullContext', enabled: state.fullContext });
}

// ---------------------------------------------------------------------------
// Flatten diff data into a linear list
// ---------------------------------------------------------------------------
function flattenLines() {
  const flat = [];
  const fileStarts = [];
  const hunkStarts = [];

  const files = state.viewScope === 'single'
    ? [state.files[state.singleFileIdx]].filter(Boolean)
    : state.files;
  const fileIndices = state.viewScope === 'single'
    ? [state.singleFileIdx]
    : state.files.map((_, i) => i);

  for (let fi = 0; fi < files.length; fi++) {
    const file = files[fi];
    const realIdx = fileIndices[fi];
    fileStarts.push(flat.length);

    flat.push({
      type: 'file-header',
      fileIdx: realIdx,
      data: file,
    });

    for (let hi = 0; hi < file.hunks.length; hi++) {
      const hunk = file.hunks[hi];
      if (hi > 0) {
        flat.push({ type: 'hunk-sep', fileIdx: realIdx, hunkIdx: hi });
      }
      hunkStarts.push(flat.length);

      for (let li = 0; li < hunk.lines.length; li++) {
        flat.push({
          type: 'line',
          fileIdx: realIdx,
          hunkIdx: hi,
          lineIdx: li,
          data: hunk.lines[li],
        });
      }
    }
  }

  state.flatLines = flat;
  state.fileStarts = fileStarts;
  state.hunkStarts = hunkStarts;
  state.changeGroupStarts = computeChangeGroupStarts(flat);

  // Pre-compute line offsets for virtual rendering
  computeLineOffsets();

  // Clamp cursor
  if (state.cursorLine >= flat.length) {
    state.cursorLine = Math.max(0, flat.length - 1);
  }
}

// Pre-compute cumulative y-offsets for each line (for virtual rendering)
function computeLineOffsets() {
  const offsets = [];
  let offset = 0;
  for (const item of state.flatLines) {
    offsets.push(offset);
    if (item.type === 'file-header') {
      offset += HEADER_HEIGHT;
    } else if (item.type === 'hunk-sep') {
      offset += HUNK_SEP_HEIGHT;
    } else {
      offset += LINE_HEIGHT;
    }
  }
  state.lineOffsets = offsets;
  state.totalHeight = offset;
}

// Get the height of a line item
function getLineHeight(item) {
  if (item.type === 'file-header') return HEADER_HEIGHT;
  if (item.type === 'hunk-sep') return HUNK_SEP_HEIGHT;
  return LINE_HEIGHT;
}

// Binary search to find first visible line at given scroll position
function findFirstVisibleLine(scrollTop) {
  const offsets = state.lineOffsets;
  if (offsets.length === 0) return 0;

  let lo = 0, hi = offsets.length - 1;
  while (lo < hi) {
    const mid = (lo + hi) >> 1;
    const lineBottom = offsets[mid] + getLineHeight(state.flatLines[mid]);
    if (lineBottom <= scrollTop) {
      lo = mid + 1;
    } else {
      hi = mid;
    }
  }
  return lo;
}

// Binary search to find last visible line at given scroll position + viewport height
function findLastVisibleLine(scrollTop, viewportHeight) {
  const offsets = state.lineOffsets;
  if (offsets.length === 0) return 0;

  const target = scrollTop + viewportHeight;
  let lo = 0, hi = offsets.length - 1;
  while (lo < hi) {
    const mid = (lo + hi + 1) >> 1;
    if (offsets[mid] < target) {
      lo = mid;
    } else {
      hi = mid - 1;
    }
  }
  return lo;
}

// Compute change group starts (like TUI's change_group_starts).
// A change group starts when a line is added/deleted and the previous line is NOT.
function computeChangeGroupStarts(flat) {
  const starts = [];
  for (let i = 0; i < flat.length; i++) {
    const item = flat[i];
    if (item.type !== 'line') continue;
    const isChange = item.data.kind === 'added' || item.data.kind === 'deleted';
    if (!isChange) continue;

    // Check if previous line was a change
    let prevIsChange = false;
    if (i > 0) {
      const prev = flat[i - 1];
      if (prev.type === 'line') {
        prevIsChange = prev.data.kind === 'added' || prev.data.kind === 'deleted';
      }
    }

    if (!prevIsChange) {
      starts.push(i);
    }
  }
  return starts;
}

// Focus the first change group (like TUI does on startup)
function focusFirstChangeGroup() {
  if (state.changeGroupStarts.length > 0) {
    state.cursorLine = state.changeGroupStarts[0];
  } else if (state.flatLines.length > 0) {
    // Fallback: find first content line
    for (let i = 0; i < state.flatLines.length; i++) {
      if (state.flatLines[i].type === 'line') {
        state.cursorLine = i;
        break;
      }
    }
  }
}

// ---------------------------------------------------------------------------
// Rendering
// ---------------------------------------------------------------------------
const STATUS_LABELS = {
  modified: 'Modified', added: 'Added', deleted: 'Deleted',
  renamed: 'Renamed', untracked: 'Untracked',
};
const STATUS_ABBREV = {
  modified: 'M', added: 'A', deleted: 'D',
  renamed: 'R', untracked: '?',
};

function renderAll() {
  __gdPerf.startRender();
  renderTree();
  renderDiff(true); // Force full render when data changes
  renderStatus();
  __gdPerf.endRender();
}

// ---------- Tree ----------
function renderTree() {
  if (!state.treeVisible) {
    treeEl.classList.add('hidden');
    return;
  }
  treeEl.classList.remove('hidden');

  const visible = getVisibleTree();
  let html = '';
  for (let i = 0; i < visible.length; i++) {
    const entry = visible[i];
    const isActive = i === state.treeCursor;
    const isFocused = state.treeFocused;
    const indent = '&nbsp;'.repeat(entry.depth * 3);
    const cls = [
      'tree-entry',
      entry.is_dir ? 'dir' : '',
      isActive ? (isFocused ? 'active' : 'active unfocused') : '',
    ].filter(Boolean).join(' ');

    let statusHtml = '';
    if (entry.status && !entry.is_dir) {
      const abbr = STATUS_ABBREV[entry.status] || '';
      statusHtml = `<span class="tree-status status-${entry.status}">${abbr}</span>`;
    }

    const icon = entry.icon || (entry.is_dir
      ? (entry.collapsed ? '&#xf4d8;' : '&#xf413;')
      : '&#xf15b;');
    const iconStyle = entry.icon_color ? `style="color: ${entry.icon_color}"` : '';

    html += `<div class="${cls}" data-tree-idx="${i}">`;
    html += `<span class="tree-guide">${indent}</span>`;
    html += `<span class="tree-icon" ${iconStyle}>${icon}</span>`;
    html += statusHtml;
    html += `<span class="tree-label">${escapeHtml(entry.label)}</span>`;
    html += `</div>`;
  }
  treeEl.innerHTML = html;

  // Scroll active into view
  const activeEl = treeEl.querySelector('.tree-entry.active');
  if (activeEl) activeEl.scrollIntoView({ block: 'nearest' });
}

function getVisibleTree() {
  const result = [];
  let skipDepth = null;
  for (const entry of state.tree) {
    if (skipDepth !== null && entry.depth > skipDepth) continue;
    skipDepth = null;

    const dirKey = treeDirKey(entry);
    // Collapsed if: explicitly in collapsedDirs, or server-collapsed and not explicitly expanded
    const isExplicitlyExpanded = state.expandedDirs && state.expandedDirs.has(dirKey);
    const isCollapsed = entry.is_dir && (state.collapsedDirs.has(dirKey) || (entry.collapsed && !isExplicitlyExpanded));
    result.push({ ...entry, collapsed: isCollapsed });
    if (isCollapsed) {
      skipDepth = entry.depth;
    }
  }
  return result;
}

function treeDirKey(entry) {
  return `${entry.depth}:${entry.label}`;
}

// Initialize expandedDirs with all directories so tree starts fully expanded
function initExpandedDirs(tree) {
  const set = new Set();
  for (const entry of tree) {
    if (entry.is_dir) {
      set.add(treeDirKey(entry));
    }
  }
  return set;
}

// ---------- Diff (Virtual Rendering) ----------
// Virtual rendering only creates DOM elements for visible lines + buffer,
// dramatically reducing DOM node count and improving performance on large diffs.

function renderDiff(forceFullRender = false) {
  const flat = state.flatLines;
  if (flat.length === 0) {
    diffPane.innerHTML = '';
    return;
  }

  // Ensure we have a content wrapper for absolute positioning
  let content = diffPane.querySelector('.diff-content');
  let spacer = diffPane.querySelector('.diff-spacer');

  if (!content || forceFullRender) {
    diffPane.innerHTML = '';
    spacer = document.createElement('div');
    spacer.className = 'diff-spacer';
    spacer.style.height = state.totalHeight + 'px';
    spacer.style.position = 'relative';

    content = document.createElement('div');
    content.className = 'diff-content';
    content.style.position = 'absolute';
    content.style.left = '0';
    content.style.right = '0';
    content.style.top = '0';

    spacer.appendChild(content);
    diffPane.appendChild(spacer);
  } else {
    // Update spacer height if content changed
    spacer.style.height = state.totalHeight + 'px';
  }

  // Calculate visible range
  const scrollTop = diffPane.scrollTop;
  const viewportHeight = diffPane.clientHeight;

  let start = findFirstVisibleLine(scrollTop);
  let end = findLastVisibleLine(scrollTop, viewportHeight);

  // Add buffer
  start = Math.max(0, start - BUFFER_LINES);
  end = Math.min(flat.length - 1, end + BUFFER_LINES);

  // Also include file header for current file if not already in range
  // (needed for sticky header behavior)
  const currentFileStart = findFileStartForLine(state.cursorLine);
  if (currentFileStart >= 0 && currentFileStart < start) {
    start = currentFileStart;
  }

  // Skip re-render if range hasn't changed significantly (optimization)
  const prev = state.renderedRange;
  if (!forceFullRender &&
      Math.abs(prev.start - start) < 5 &&
      Math.abs(prev.end - end) < 5 &&
      start >= prev.start && end <= prev.end) {
    // Just update cursor and visual selection classes
    updateCursorClasses();
    return;
  }

  state.renderedRange = { start, end };

  // Compute visual selection range
  const hasVisualSelection = state.visualAnchor !== null;
  const visualLo = hasVisualSelection ? Math.min(state.visualAnchor, state.cursorLine) : -1;
  const visualHi = hasVisualSelection ? Math.max(state.visualAnchor, state.cursorLine) : -1;

  // Render only visible lines
  const frag = document.createDocumentFragment();

  for (let i = start; i <= end; i++) {
    const item = flat[i];
    const top = state.lineOffsets[i];
    const height = getLineHeight(item);

    if (item.type === 'file-header') {
      const div = document.createElement('div');
      div.className = 'file-header';
      div.dataset.flatIdx = i;
      div.style.position = 'absolute';
      div.style.left = '0';
      div.style.right = '0';
      div.style.top = top + 'px';
      div.style.height = height + 'px';
      const file = item.data;
      const label = STATUS_LABELS[file.status] || file.status;
      div.innerHTML = `${escapeHtml(file.path)}<span class="file-status">(${label})</span>`;
      // Use closure to capture fileIdx
      const fileIdx = item.fileIdx;
      div.addEventListener('click', () => {
        state.singleFileIdx = fileIdx;
        state.viewScope = 'single';
        state.cursorLine = 0;
        flattenLines();
        renderAll();
      });
      frag.appendChild(div);
    } else if (item.type === 'hunk-sep') {
      const div = document.createElement('div');
      div.className = 'hunk-sep';
      div.style.position = 'absolute';
      div.style.left = '0';
      div.style.right = '0';
      div.style.top = top + 'px';
      div.style.height = height + 'px';
      frag.appendChild(div);
    } else {
      const line = item.data;
      const div = document.createElement('div');
      const kindCls = line.kind === 'added' ? 'line-added'
        : line.kind === 'deleted' ? 'line-deleted' : '';
      div.className = `diff-line ${kindCls}`;
      if (i === state.cursorLine) div.classList.add('cursor-line');
      if (hasVisualSelection && i >= visualLo && i <= visualHi) div.classList.add('visual-selected');
      div.dataset.flatIdx = i;
      div.style.position = 'absolute';
      div.style.left = '0';
      div.style.right = '0';
      div.style.top = top + 'px';
      div.style.height = height + 'px';

      const marker = line.kind === 'added' ? '+' : line.kind === 'deleted' ? '-' : ' ';
      const oldNo = line.old_lineno != null ? line.old_lineno : '';
      const newNo = line.new_lineno != null ? line.new_lineno : '';

      div.innerHTML =
        `<div class="gutter">` +
          `<span class="gutter-old">${oldNo}</span>` +
          `<span class="gutter-sep">│</span>` +
          `<span class="gutter-new">${newNo}</span>` +
          `<span class="gutter-sep">│</span>` +
        `</div>` +
        `<div class="marker">${marker}</div>` +
        `<div class="line-content">${line.content_html || escapeHtml(line.raw_content)}</div>`;

      frag.appendChild(div);
    }
  }

  content.innerHTML = '';
  content.appendChild(frag);
}

// Find the file-header line that contains a given line index
function findFileStartForLine(lineIdx) {
  for (let i = state.fileStarts.length - 1; i >= 0; i--) {
    if (state.fileStarts[i] <= lineIdx) {
      return state.fileStarts[i];
    }
  }
  return 0;
}

// Update only cursor/visual classes without full re-render
function updateCursorClasses() {
  const content = diffPane.querySelector('.diff-content');
  if (!content) return;

  const hasVisualSelection = state.visualAnchor !== null;
  const visualLo = hasVisualSelection ? Math.min(state.visualAnchor, state.cursorLine) : -1;
  const visualHi = hasVisualSelection ? Math.max(state.visualAnchor, state.cursorLine) : -1;

  // Remove old cursor
  const oldCursor = content.querySelector('.cursor-line');
  if (oldCursor) oldCursor.classList.remove('cursor-line');

  // Update visual selection and cursor
  const lines = content.querySelectorAll('[data-flat-idx]');
  for (const el of lines) {
    const idx = parseInt(el.dataset.flatIdx, 10);
    el.classList.toggle('cursor-line', idx === state.cursorLine);
    el.classList.toggle('visual-selected', hasVisualSelection && idx >= visualLo && idx <= visualHi);
  }
}

// Handle scroll events for virtual rendering
function onDiffScroll() {
  if (state.scrollScheduled) return;
  state.scrollScheduled = true;
  requestAnimationFrame(() => {
    state.scrollScheduled = false;
    renderDiff();
  });
}

function scrollCursorIntoView(center = false) {
  // Use requestAnimationFrame to avoid forced reflow
  requestAnimationFrame(() => {
    // Ensure the cursor line is rendered
    const cursorOffset = state.lineOffsets[state.cursorLine];
    if (cursorOffset === undefined) return;

    const cursorHeight = getLineHeight(state.flatLines[state.cursorLine]);
    const scrollTop = diffPane.scrollTop;
    const viewportHeight = diffPane.clientHeight;

    // Account for sticky header and status bar
    const headerBuffer = 40;
    const footerBuffer = 24;

    if (center) {
      // Center the cursor line
      const targetScroll = cursorOffset - (viewportHeight / 2) + (cursorHeight / 2);
      diffPane.scrollTop = Math.max(0, targetScroll);
    } else {
      // Scroll minimally to make cursor visible
      if (cursorOffset < scrollTop + headerBuffer) {
        // Cursor above viewport
        diffPane.scrollTop = Math.max(0, cursorOffset - headerBuffer);
      } else if (cursorOffset + cursorHeight > scrollTop + viewportHeight - footerBuffer) {
        // Cursor below viewport
        diffPane.scrollTop = cursorOffset + cursorHeight - viewportHeight + footerBuffer;
      }
    }

    // Re-render to ensure cursor line is visible
    renderDiff();
  });
}

// ---------- Status bar ----------
function renderStatus() {
  let left = '';
  if (state.visualAnchor !== null) {
    left = '<span style="color: var(--fg-search-match)">-- VISUAL --</span>';
  } else if (state.viewScope === 'single') {
    const file = state.files[state.singleFileIdx];
    if (file) {
      const total = state.files.length;
      const idx = state.singleFileIdx + 1;
      left = `<span style="opacity:0.5">${escapeHtml(file.path)}</span> <span>‹ ${idx}/${total} ›</span>`;
    }
  } else {
    left = `${state.files.length} file${state.files.length !== 1 ? 's' : ''}`;
  }
  statusLeft.innerHTML = left;

  // Position indicator (theme toggle is static in HTML)
  const total = state.flatLines.length;
  const pos = total === 0 ? '' :
    state.cursorLine === 0 ? 'TOP' :
    state.cursorLine >= total - 1 ? 'END' :
    Math.round((state.cursorLine / (total - 1)) * 100) + '%';

  // Update position span, preserve theme toggle button
  const posSpan = statusRight.querySelector('#status-position') || (() => {
    const span = document.createElement('span');
    span.id = 'status-position';
    statusRight.insertBefore(span, themeToggle);
    return span;
  })();
  posSpan.textContent = pos;
}

// ---------------------------------------------------------------------------
// Keyboard navigation
// ---------------------------------------------------------------------------
function moveCursor(delta) {
  const newPos = Math.max(0, Math.min(state.flatLines.length - 1, state.cursorLine + delta));
  setCursor(newPos, delta < 0 ? 'backward' : 'forward');
  syncTreeCursor();
}

function setCursor(pos, direction = 'forward', center = false) {
  const oldCursor = state.cursorLine;
  const requestedPos = pos; // Remember original request for top/bottom detection
  state.cursorLine = Math.max(0, Math.min(state.flatLines.length - 1, pos));

  // Skip headers/separators in the direction of movement
  const item = state.flatLines[state.cursorLine];
  if (item && item.type !== 'line') {
    let found = false;
    if (direction === 'backward') {
      // Moving up: search backward for content line
      for (let i = state.cursorLine - 1; i >= 0; i--) {
        if (state.flatLines[i].type === 'line') {
          state.cursorLine = i;
          found = true;
          break;
        }
      }
    } else {
      // Moving down or jumping: search forward for content line
      for (let i = state.cursorLine + 1; i < state.flatLines.length; i++) {
        if (state.flatLines[i].type === 'line') {
          state.cursorLine = i;
          found = true;
          break;
        }
      }
    }
    // Fallback: search opposite direction if nothing found in primary direction
    if (!found) {
      if (direction === 'backward') {
        for (let i = state.cursorLine + 1; i < state.flatLines.length; i++) {
          if (state.flatLines[i].type === 'line') {
            state.cursorLine = i;
            break;
          }
        }
      } else {
        for (let i = state.cursorLine - 1; i >= 0; i--) {
          if (state.flatLines[i].type === 'line') {
            state.cursorLine = i;
            break;
          }
        }
      }
    }
  }

  // With virtual rendering, we compute scroll position from line offsets
  // rather than relying on DOM elements existing
  const cursorOffset = state.lineOffsets[state.cursorLine];
  const cursorHeight = getLineHeight(state.flatLines[state.cursorLine]);
  const scrollTop = diffPane.scrollTop;
  const viewportHeight = diffPane.clientHeight;
  const headerBuffer = 40; // account for sticky headers
  const footerBuffer = 24; // account for status bar

  if (requestedPos <= 0) {
    // Going to top: scroll to absolute top
    diffPane.scrollTop = 0;
  } else if (requestedPos >= state.flatLines.length - 1) {
    // Going to bottom: ensure last line is visible
    diffPane.scrollTop = Math.max(0, state.totalHeight - viewportHeight + footerBuffer);
  } else if (center) {
    // Center the cursor line in viewport
    const targetScroll = cursorOffset - (viewportHeight / 2) + (cursorHeight / 2);
    diffPane.scrollTop = Math.max(0, targetScroll);
  } else {
    // Scroll minimally to make cursor visible (nearest behavior)
    if (cursorOffset < scrollTop + headerBuffer) {
      // Cursor above viewport
      diffPane.scrollTop = Math.max(0, cursorOffset - headerBuffer);
    } else if (cursorOffset + cursorHeight > scrollTop + viewportHeight - footerBuffer) {
      // Cursor below viewport
      diffPane.scrollTop = cursorOffset + cursorHeight - viewportHeight + footerBuffer;
    }
  }

  // Re-render to show updated cursor position
  // Visual selection requires full update to set classes correctly
  if (state.visualAnchor !== null && state.cursorLine !== oldCursor) {
    renderDiff();
  } else {
    renderDiff();
  }

  renderStatus();
  syncTreeCursor();
}

function pageHeight() {
  return Math.floor(diffPane.clientHeight / 20); // 20px line height
}

// Get the file index for the current cursor position
function currentFileIdx() {
  for (let i = state.fileStarts.length - 1; i >= 0; i--) {
    if (state.fileStarts[i] <= state.cursorLine) {
      return state.flatLines[state.fileStarts[i]]?.fileIdx ?? 0;
    }
  }
  return 0;
}

function jumpNextHunk() {
  // Match TUI's nav_du_down + reducer logic:
  // 1. Find next change group > cursor
  // 2. If none found AND in single mode, advance to next file's first change group
  // 3. If none found AND in all mode, stay put
  const targets = state.changeGroupStarts;
  for (const t of targets) {
    if (t > state.cursorLine) {
      setCursor(t, 'forward', true); // center cursor like TUI
      return;
    }
  }
  // No more change groups found in current view
  if (state.viewScope === 'single') {
    // Single-file mode: advance to next file if available
    if (state.singleFileIdx < state.files.length - 1) {
      state.singleFileIdx++;
      flattenLines();
      // Jump to first change group of new file
      if (state.changeGroupStarts.length > 0) {
        state.cursorLine = state.changeGroupStarts[0];
      } else {
        state.cursorLine = 0;
      }
      renderAll();
      syncTreeCursor();
      centerCursor(); // center after file transition
    }
    // else: last file, stay put (do nothing)
  }
  // else: all mode, stay put (do nothing - matches TUI)
}

function jumpPrevHunk() {
  // Match TUI's nav_du_up + reducer logic:
  // 1. Find previous change group < cursor
  // 2. If none found AND in single mode, retreat to previous file's last change group
  // 3. If none found AND in all mode, stay put

  const targets = state.changeGroupStarts;

  // Try to find a previous change group in current view
  for (let i = targets.length - 1; i >= 0; i--) {
    if (targets[i] < state.cursorLine) {
      setCursor(targets[i], 'backward', true); // center cursor like TUI
      return;
    }
  }

  // No previous change group found in current view
  if (state.viewScope === 'single') {
    // Single-file mode: retreat to previous file if available
    if (state.singleFileIdx > 0) {
      state.singleFileIdx--;
      flattenLines();
      // Jump to last change group of new file
      if (state.changeGroupStarts.length > 0) {
        state.cursorLine = state.changeGroupStarts[state.changeGroupStarts.length - 1];
      } else {
        state.cursorLine = Math.max(0, state.flatLines.length - 1);
      }
      renderAll();
      syncTreeCursor();
      centerCursor(); // center after file transition
    }
    // else: first file, stay put (do nothing)
  }
  // else: all mode, stay put (do nothing - matches TUI)
}

function jumpNextFile() {
  if (state.viewScope === 'single') {
    if (state.singleFileIdx < state.files.length - 1) {
      state.singleFileIdx++;
      state.cursorLine = 0;
      flattenLines();
      renderAll();
      syncTreeCursor();
    }
    return;
  }
  for (const fs of state.fileStarts) {
    if (fs > state.cursorLine) {
      setCursor(fs);
      return;
    }
  }
}

function jumpPrevFile() {
  if (state.viewScope === 'single') {
    if (state.singleFileIdx > 0) {
      state.singleFileIdx--;
      state.cursorLine = 0;
      flattenLines();
      renderAll();
      syncTreeCursor();
    }
    return;
  }
  for (let i = state.fileStarts.length - 1; i >= 0; i--) {
    if (state.fileStarts[i] < state.cursorLine) {
      setCursor(state.fileStarts[i]);
      return;
    }
  }
}

function toggleSingleFile() {
  if (state.viewScope === 'single') {
    state.viewScope = 'all';
  } else {
    // Find which file the cursor is in
    const item = state.flatLines[state.cursorLine];
    if (item) state.singleFileIdx = item.fileIdx;
    state.viewScope = 'single';
  }
  flattenLines();
  focusFirstChangeGroup();
  renderAll();
}

function toggleTree() {
  state.treeVisible = !state.treeVisible;
  renderTree();
}

function toggleTreeFocus() {
  if (!state.treeVisible) {
    state.treeVisible = true;
  }
  state.treeFocused = !state.treeFocused;
  renderTree();
}

function centerCursor() {
  if (state.flatLines.length === 0) return;
  const cursorOffset = state.lineOffsets[state.cursorLine];
  const cursorHeight = getLineHeight(state.flatLines[state.cursorLine]);
  const viewportHeight = diffPane.clientHeight;
  const targetScroll = cursorOffset - (viewportHeight / 2) + (cursorHeight / 2);
  diffPane.scrollTop = Math.max(0, targetScroll);
  renderDiff();
}

// ---------- Tree navigation ----------
function treeMoveCursor(delta) {
  const visible = getVisibleTree();
  state.treeCursor = Math.max(0, Math.min(visible.length - 1, state.treeCursor + delta));
  renderTree();
}

function treeSelect() {
  const visible = getVisibleTree();
  const entry = visible[state.treeCursor];
  if (!entry) return;

  if (entry.is_dir) {
    treeToggleCollapse();
  } else if (entry.file_idx != null) {
    state.singleFileIdx = entry.file_idx;
    state.viewScope = 'single';
    flattenLines();
    focusFirstChangeGroup();
    renderAll();
  }
}

function treeToggleCollapse() {
  const visible = getVisibleTree();
  const entry = visible[state.treeCursor];
  if (!entry || !entry.is_dir) return;

  const key = treeDirKey(entry);
  // Check if currently collapsed (either via state or server default)
  const isCollapsed = state.collapsedDirs.has(key) || entry.collapsed;
  if (isCollapsed) {
    // Expand: remove from collapsedDirs (this overrides server collapsed)
    state.collapsedDirs.delete(key);
    // If it was collapsed from server, we need to mark it explicitly expanded
    // by NOT having it in collapsedDirs. But we also need to track expansions.
    if (!state.expandedDirs) state.expandedDirs = new Set();
    state.expandedDirs.add(key);
  } else {
    // Collapse: add to collapsedDirs
    state.collapsedDirs.add(key);
    if (state.expandedDirs) state.expandedDirs.delete(key);
  }
  renderTree();
}

function treeJumpTop() {
  state.treeCursor = 0;
  renderTree();
}

function treeJumpBottom() {
  const visible = getVisibleTree();
  state.treeCursor = Math.max(0, visible.length - 1);
  renderTree();
}

function treeToggleCollapseRecursive() {
  const visible = getVisibleTree();
  const entry = visible[state.treeCursor];
  if (!entry || !entry.is_dir) return;

  if (!state.expandedDirs) state.expandedDirs = new Set();

  const currentKey = treeDirKey(entry);
  const currentDepth = entry.depth;
  // Check if currently collapsed (either via state or server default)
  const isCurrentlyCollapsed = state.collapsedDirs.has(currentKey) ||
    (entry.collapsed && !state.expandedDirs.has(currentKey));
  const shouldExpand = isCurrentlyCollapsed;

  // Find all nested directories (including current)
  // Walk through visible tree from current position
  const visibleIdx = state.treeCursor;
  for (let i = visibleIdx; i < visible.length; i++) {
    const e = visible[i];
    // Stop when we reach an entry at same or shallower depth (except first iteration)
    if (i > visibleIdx && e.depth <= currentDepth) break;

    if (e.is_dir) {
      const key = treeDirKey(e);
      if (shouldExpand) {
        state.collapsedDirs.delete(key);
        state.expandedDirs.add(key);
      } else {
        state.collapsedDirs.add(key);
        state.expandedDirs.delete(key);
      }
    }
  }

  // Also need to handle dirs in the original tree that may be hidden
  // by collapse. Walk the full tree to find nested dirs.
  const baseDepth = entry.depth;
  let inSubtree = false;
  for (const e of state.tree) {
    if (e.depth === baseDepth && e.label === entry.label) {
      inSubtree = true;
    } else if (inSubtree && e.depth <= baseDepth) {
      inSubtree = false;
    }

    if (inSubtree && e.is_dir) {
      const key = treeDirKey(e);
      if (shouldExpand) {
        state.collapsedDirs.delete(key);
        state.expandedDirs.add(key);
      } else {
        state.collapsedDirs.add(key);
        state.expandedDirs.delete(key);
      }
    }
  }

  renderTree();
}

function syncTreeCursor() {
  if (!state.treeVisible || state.treeFocused) return;
  const item = state.flatLines[state.cursorLine];
  if (!item || item.fileIdx == null) return;
  const visible = getVisibleTree();
  for (let i = 0; i < visible.length; i++) {
    if (!visible[i].is_dir && visible[i].file_idx === item.fileIdx) {
      if (state.treeCursor !== i) {
        state.treeCursor = i;
        renderTree();
      }
      return;
    }
  }
}

// ---------- Search ----------
function openSearch() {
  state.searchActive = true;
  searchBar.classList.add('visible');
  searchInput.value = state.searchQuery;
  searchInput.focus();
  searchInput.select();
}

function closeSearch() {
  state.searchActive = false;
  searchBar.classList.remove('visible');
  searchInput.blur();
  state.searchMatches = [];
  state.searchCurrentIdx = -1;
  searchCount.textContent = '';
  // Remove search highlighting
  diffPane.querySelectorAll('.search-match').forEach(el => {
    el.outerHTML = el.innerHTML;
  });
}

function submitSearch() {
  const query = searchInput.value.trim();
  if (!query) {
    closeSearch();
    return;
  }
  state.searchQuery = query;
  performSearch();
  if (state.searchMatches.length > 0) {
    state.searchCurrentIdx = 0;
    scrollToMatch(0);
  }
  updateSearchCount();
  searchInput.blur();
}

function performSearch() {
  state.searchMatches = [];
  const query = state.searchQuery.toLowerCase();

  for (let i = 0; i < state.flatLines.length; i++) {
    const item = state.flatLines[i];
    if (item.type === 'line' && item.data.raw_content.toLowerCase().includes(query)) {
      state.searchMatches.push(i);
    }
  }
}

function scrollToMatch(matchIdx) {
  if (matchIdx < 0 || matchIdx >= state.searchMatches.length) return;
  state.searchCurrentIdx = matchIdx;
  const flatIdx = state.searchMatches[matchIdx];
  setCursor(flatIdx);
  updateSearchCount();
}

function nextMatch() {
  if (state.searchMatches.length === 0) return;
  const next = (state.searchCurrentIdx + 1) % state.searchMatches.length;
  scrollToMatch(next);
}

function prevMatch() {
  if (state.searchMatches.length === 0) return;
  const prev = (state.searchCurrentIdx - 1 + state.searchMatches.length) % state.searchMatches.length;
  scrollToMatch(prev);
}

function updateSearchCount() {
  if (state.searchMatches.length === 0) {
    searchCount.textContent = 'No matches';
  } else {
    searchCount.textContent = `${state.searchCurrentIdx + 1} / ${state.searchMatches.length}`;
  }
}

// ---------- Help ----------
function toggleHelp() {
  state.helpVisible = !state.helpVisible;
  helpOverlay.classList.toggle('visible', state.helpVisible);
}

// ---------- Visual Selection ----------
function toggleVisualSelect() {
  if (state.visualAnchor !== null) {
    state.visualAnchor = null;
  } else {
    state.visualAnchor = state.cursorLine;
  }
  renderDiff();
  renderStatus();
}

function resolveLineno(flatIdx) {
  const item = state.flatLines[flatIdx];
  if (!item || item.type !== 'line') return null;
  // Prefer new_lineno (for added/context), fall back to old_lineno (for deleted)
  return item.data.new_lineno ?? item.data.old_lineno ?? null;
}

function yankSelection() {
  if (state.visualAnchor === null) return;

  const lo = Math.min(state.visualAnchor, state.cursorLine);
  const hi = Math.max(state.visualAnchor, state.cursorLine);

  // Get path from first selected line
  const item = state.flatLines[lo];
  let path = '';
  if (item?.data?.path) {
    path = item.data.path;
  } else if (item?.fileIdx != null && state.files[item.fileIdx]) {
    path = state.files[item.fileIdx].path;
  }

  const loLine = resolveLineno(lo);
  const hiLine = resolveLineno(hi);

  // Format the reference string
  let ref;
  if (loLine != null && hiLine != null) {
    ref = loLine === hiLine ? `${path}:${loLine}` : `${path}:${loLine}-${hiLine}`;
  } else {
    ref = path;
  }

  navigator.clipboard.writeText(ref);
  state.visualAnchor = null;
  renderDiff();
  renderStatus();
}

// ---------- Staging ----------
// Helper: get info about the current cursor line
function currentLineInfo() {
  const item = state.flatLines[state.cursorLine];
  if (!item) return null;
  return {
    type: item.type,
    fileIdx: item.fileIdx,
    hunkIdx: item.hunkIdx,
    lineIdx: item.data?.line_idx ?? item.lineIdx,
    kind: item.data?.kind,
  };
}

function stageCurrentLine() {
  const info = currentLineInfo();
  if (!info || info.type !== 'line') return;
  // Can't stage context lines (mirrors TUI behavior)
  if (info.kind === 'context') return;
  sendMessage({
    type: 'StageLine',
    file_idx: info.fileIdx,
    hunk_idx: info.hunkIdx,
    line_idx: info.lineIdx,
  });
}

function stageCurrentHunk() {
  const info = currentLineInfo();
  if (!info) return;
  // Can't stage from file header (no hunk context)
  if (info.type === 'file-header') return;
  // For hunk-sep or line, hunkIdx is valid
  if (info.hunkIdx == null) return;
  sendMessage({
    type: 'StageHunk',
    file_idx: info.fileIdx,
    hunk_idx: info.hunkIdx,
  });
}

// ---------------------------------------------------------------------------
// Event handlers
// ---------------------------------------------------------------------------
const NAV_KEYS = new Set(['j', 'k', 'ArrowDown', 'ArrowUp', 'd', 'u', 'g', 'G', 'Home', 'End', ']', '[', '}', '{', 'n', 'N']);

document.addEventListener('keydown', (e) => {
  const navStart = performance.now();

  // Search input mode
  if (state.searchActive && document.activeElement === searchInput) {
    if (e.key === 'Enter') {
      e.preventDefault();
      submitSearch();
    } else if (e.key === 'Escape') {
      e.preventDefault();
      closeSearch();
    }
    return;
  }

  // Help overlay
  if (state.helpVisible) {
    if (e.key === '?' || e.key === 'Escape' || e.key === 'q') {
      e.preventDefault();
      toggleHelp();
    }
    return;
  }

  // Tree-focused keys
  if (state.treeFocused) {
    // Handle pending z key for za/zA collapse
    if (state.pendingTreeKey === 'z') {
      state.pendingTreeKey = null;
      if (e.key === 'a') {
        e.preventDefault();
        treeToggleCollapse();
        return;
      } else if (e.key === 'A') {
        e.preventDefault();
        treeToggleCollapseRecursive();
        return;
      }
      // Fall through to normal handling for other keys
    }

    switch (e.key) {
      case 'j': case 'ArrowDown': e.preventDefault(); treeMoveCursor(1); return;
      case 'k': case 'ArrowUp': e.preventDefault(); treeMoveCursor(-1); return;
      case 'g': case 'Home': e.preventDefault(); treeJumpTop(); return;
      case 'G': case 'End': e.preventDefault(); treeJumpBottom(); return;
      case 'Enter': case ' ': e.preventDefault(); treeSelect(); return;
      case 't': e.preventDefault(); toggleTreeFocus(); return;
      case 'l': e.preventDefault(); toggleTree(); return;
      case 'z': e.preventDefault(); state.pendingTreeKey = 'z'; return;
      case '?': e.preventDefault(); toggleHelp(); return;
    }
    // Fall through for other keys
  }

  switch (e.key) {
    // Navigation
    case 'j': case 'ArrowDown': e.preventDefault(); moveCursor(1); break;
    case 'k': case 'ArrowUp': e.preventDefault(); moveCursor(-1); break;
    case 'd': e.preventDefault(); moveCursor(Math.floor(pageHeight() / 2)); break;
    case 'u': e.preventDefault(); moveCursor(-Math.floor(pageHeight() / 2)); break;
    case 'g': if (!e.ctrlKey) { e.preventDefault(); setCursor(0); } break;
    case 'G': e.preventDefault(); setCursor(state.flatLines.length - 1); break;
    case 'Home': e.preventDefault(); setCursor(0); break;
    case 'End': e.preventDefault(); setCursor(state.flatLines.length - 1); break;
    case 'z': e.preventDefault(); centerCursor(); break;

    // Diff navigation
    case ']': e.preventDefault(); jumpNextHunk(); break;
    case '[': e.preventDefault(); jumpPrevHunk(); break;
    case '}': e.preventDefault(); jumpNextFile(); break;
    case '{': e.preventDefault(); jumpPrevFile(); break;
    case 's': e.preventDefault(); toggleSingleFile(); break;
    case 'o': e.preventDefault(); toggleFullContext(); break;

    // Tree
    case 'l': e.preventDefault(); toggleTree(); break;
    case 't': e.preventDefault(); toggleTreeFocus(); break;

    // Staging
    case 'a': e.preventDefault(); stageCurrentLine(); break;
    case 'A': e.preventDefault(); stageCurrentHunk(); break;

    // Visual selection
    case 'v': e.preventDefault(); toggleVisualSelect(); break;
    case 'y': e.preventDefault(); yankSelection(); break;

    // Search
    case '/': e.preventDefault(); openSearch(); break;
    case 'n': e.preventDefault(); nextMatch(); break;
    case 'N': e.preventDefault(); prevMatch(); break;
    case 'Escape':
      if (state.visualAnchor !== null) {
        state.visualAnchor = null;
        renderDiff();
        renderStatus();
      } else if (state.searchActive) {
        closeSearch();
      }
      break;

    // Help
    case '?': e.preventDefault(); toggleHelp(); break;

    // Theme toggle
    case 'T': e.preventDefault(); cycleTheme(); break;
  }

  // Record navigation timing for perf analysis
  if (NAV_KEYS.has(e.key)) {
    __gdPerf.recordNavigation(e.key, performance.now() - navStart);
  }
});

// Tree click handler
treeEl.addEventListener('click', (e) => {
  const entry = e.target.closest('.tree-entry');
  if (!entry) return;
  const idx = parseInt(entry.dataset.treeIdx, 10);
  state.treeCursor = idx;

  const visible = getVisibleTree();
  const item = visible[idx];
  if (!item) return;

  if (item.is_dir) {
    treeToggleCollapse();
  } else if (item.file_idx != null) {
    state.singleFileIdx = item.file_idx;
    state.viewScope = 'single';
    flattenLines();
    focusFirstChangeGroup();
    renderAll();
  }
});

// ---------------------------------------------------------------------------
// Theme toggle
// ---------------------------------------------------------------------------
const THEME_ICONS = {
  light: '\uf0235',   // sun
  dark: '\uf0238',    // moon
  system: '\uf0544',  // laptop
};

function initTheme() {
  const saved = localStorage.getItem('gd-theme');
  if (saved && ['light', 'dark', 'system'].includes(saved)) {
    state.theme = saved;
  }
  applyTheme();

  // Listen for system preference changes
  window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', () => {
    if (state.theme === 'system') {
      // Theme will be re-applied via CSS media query
    }
  });
}

function applyTheme() {
  document.documentElement.dataset.theme = state.theme;
  if (themeToggle) {
    themeToggle.innerHTML = THEME_ICONS[state.theme];
    const labels = { light: 'Light', dark: 'Dark', system: 'System' };
    themeToggle.title = `Theme: ${labels[state.theme]} (T)`;
  }
}

function cycleTheme() {
  const order = ['system', 'light', 'dark'];
  const idx = order.indexOf(state.theme);
  state.theme = order[(idx + 1) % order.length];
  localStorage.setItem('gd-theme', state.theme);
  applyTheme();
}

if (themeToggle) {
  themeToggle.addEventListener('click', cycleTheme);
}

// ---------------------------------------------------------------------------
// Tree resizing
// ---------------------------------------------------------------------------
function initTreeResize() {
  const saved = localStorage.getItem('gd-tree-width');
  if (saved) {
    const width = parseInt(saved, 10);
    if (!isNaN(width) && width >= 150 && width <= 500) {
      state.treeWidth = width;
    }
  }
  applyTreeWidth();
}

function applyTreeWidth() {
  if (treeEl) {
    treeEl.style.width = state.treeWidth + 'px';
  }
}

function onResizeStart(e) {
  e.preventDefault();
  state.resizing = true;
  resizeHandle.classList.add('dragging');
  document.body.style.cursor = 'col-resize';
  document.body.style.userSelect = 'none';
}

function onResizeMove(e) {
  if (!state.resizing) return;
  // Tree is on the right, so width = window.innerWidth - e.clientX - resize handle width
  const newWidth = Math.max(150, Math.min(500, window.innerWidth - e.clientX - 4));
  state.treeWidth = newWidth;
  applyTreeWidth();
}

function onResizeEnd() {
  if (!state.resizing) return;
  state.resizing = false;
  resizeHandle.classList.remove('dragging');
  document.body.style.cursor = '';
  document.body.style.userSelect = '';
  localStorage.setItem('gd-tree-width', state.treeWidth.toString());
}

if (resizeHandle) {
  resizeHandle.addEventListener('mousedown', onResizeStart);
  document.addEventListener('mousemove', onResizeMove);
  document.addEventListener('mouseup', onResizeEnd);
}

// ---------------------------------------------------------------------------
// Utilities
// ---------------------------------------------------------------------------
function escapeHtml(str) {
  return str.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');
}

// ---------------------------------------------------------------------------
// Init
// ---------------------------------------------------------------------------
initTheme();
initTreeResize();

// Virtual rendering: re-render on scroll
diffPane.addEventListener('scroll', onDiffScroll, { passive: true });

connect();
