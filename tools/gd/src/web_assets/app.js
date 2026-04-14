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
  pendingTreeKey: null,
  searchQuery: '',
  searchMatches: [],
  searchCurrentIdx: -1,
  searchActive: false,
  helpVisible: false,
  collapsedDirs: new Set(),
  fullContext: false,
};

// DOM refs
const treeEl = document.getElementById('tree');
const diffPane = document.getElementById('diff-pane');
const statusLeft = document.getElementById('status-left');
const statusRight = document.getElementById('status-right');
const searchBar = document.getElementById('search-bar');
const searchInput = document.getElementById('search-input');
const searchCount = document.getElementById('search-count');
const helpOverlay = document.getElementById('help-overlay');

// ---------------------------------------------------------------------------
// WebSocket
// ---------------------------------------------------------------------------
let ws = null;

function connect() {
  const proto = location.protocol === 'https:' ? 'wss:' : 'ws:';
  ws = new WebSocket(`${proto}//${location.host}/ws`);

  ws.onmessage = (ev) => {
    const msg = JSON.parse(ev.data);
    if (msg.type === 'DiffData') {
      state.files = msg.files;
      state.tree = msg.tree;
      flattenLines();
      // Focus first change group on initial load (like TUI)
      focusFirstChangeGroup();
      renderAll();
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

  // Clamp cursor
  if (state.cursorLine >= flat.length) {
    state.cursorLine = Math.max(0, flat.length - 1);
  }
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
  renderTree();
  renderDiff();
  renderStatus();
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

// ---------- Diff ----------
function renderDiff() {
  const flat = state.flatLines;
  // For performance we render into a document fragment
  const frag = document.createDocumentFragment();

  for (let i = 0; i < flat.length; i++) {
    const item = flat[i];

    if (item.type === 'file-header') {
      const div = document.createElement('div');
      div.className = 'file-header';
      div.dataset.flatIdx = i;
      const file = item.data;
      const label = STATUS_LABELS[file.status] || file.status;
      div.innerHTML = `${escapeHtml(file.path)}<span class="file-status">(${label})</span>`;
      div.addEventListener('click', () => {
        state.singleFileIdx = item.fileIdx;
        state.viewScope = 'single';
        state.cursorLine = 0;
        flattenLines();
        renderAll();
      });
      frag.appendChild(div);
    } else if (item.type === 'hunk-sep') {
      const div = document.createElement('div');
      div.className = 'hunk-sep';
      frag.appendChild(div);
    } else {
      const line = item.data;
      const div = document.createElement('div');
      const kindCls = line.kind === 'added' ? 'line-added'
        : line.kind === 'deleted' ? 'line-deleted' : '';
      div.className = `diff-line ${kindCls}`;
      if (i === state.cursorLine) div.classList.add('cursor-line');
      div.dataset.flatIdx = i;

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

  diffPane.innerHTML = '';
  diffPane.appendChild(frag);

  scrollCursorIntoView();
}

function scrollCursorIntoView() {
  const el = diffPane.querySelector('.cursor-line');
  if (el) el.scrollIntoView({ block: 'nearest' });
}

// ---------- Status bar ----------
function renderStatus() {
  let left = '';
  if (state.viewScope === 'single') {
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

  const total = state.flatLines.length;
  const pos = total === 0 ? '' :
    state.cursorLine === 0 ? 'TOP' :
    state.cursorLine >= total - 1 ? 'END' :
    Math.round((state.cursorLine / (total - 1)) * 100) + '%';
  statusRight.textContent = pos;
}

// ---------------------------------------------------------------------------
// Keyboard navigation
// ---------------------------------------------------------------------------
function moveCursor(delta) {
  const newPos = Math.max(0, Math.min(state.flatLines.length - 1, state.cursorLine + delta));
  setCursor(newPos);
  syncTreeCursor();
}

function setCursor(pos) {
  const old = diffPane.querySelector('.cursor-line');
  if (old) old.classList.remove('cursor-line');

  state.cursorLine = Math.max(0, Math.min(state.flatLines.length - 1, pos));

  // Skip headers/separators
  const item = state.flatLines[state.cursorLine];
  if (item && item.type !== 'line') {
    // Try to find the next content line
    for (let i = state.cursorLine + 1; i < state.flatLines.length; i++) {
      if (state.flatLines[i].type === 'line') {
        state.cursorLine = i;
        break;
      }
    }
  }

  const el = diffPane.querySelector(`[data-flat-idx="${state.cursorLine}"]`);
  if (el) {
    el.classList.add('cursor-line');
    el.scrollIntoView({ block: 'nearest' });
  }
  renderStatus();
  syncTreeCursor();
}

function pageHeight() {
  return Math.floor(diffPane.clientHeight / 20); // 20px line height
}

function jumpNextHunk() {
  // Use change groups (like TUI's nav_du_down) for accurate navigation
  const targets = state.changeGroupStarts;
  for (const t of targets) {
    if (t > state.cursorLine) {
      setCursor(t);
      return;
    }
  }
  // No more change groups found - advance to next file
  if (state.viewScope === 'single') {
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
    }
  } else {
    // Find next file's first change group
    for (const fs of state.fileStarts) {
      if (fs > state.cursorLine) {
        const nextFileIdx = state.flatLines[fs]?.fileIdx;
        for (const cg of state.changeGroupStarts) {
          if (cg >= fs && state.flatLines[cg]?.fileIdx === nextFileIdx) {
            setCursor(cg);
            return;
          }
        }
        // No change group found, just go to file start
        setCursor(fs);
        return;
      }
    }
  }
}

function jumpPrevHunk() {
  // Use change groups (like TUI's nav_du_up) for accurate navigation
  const targets = state.changeGroupStarts;
  for (let i = targets.length - 1; i >= 0; i--) {
    if (targets[i] < state.cursorLine) {
      setCursor(targets[i]);
      return;
    }
  }
  // No previous change group found - go to previous file's last change group
  if (state.viewScope === 'single') {
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
    }
  } else {
    // Find the start of current file
    let currentFileStart = 0;
    for (let i = state.fileStarts.length - 1; i >= 0; i--) {
      if (state.fileStarts[i] <= state.cursorLine) {
        currentFileStart = state.fileStarts[i];
        break;
      }
    }
    // Find previous file's last change group
    if (currentFileStart > 0) {
      const prevFileIdx = state.flatLines[currentFileStart - 1]?.fileIdx;
      for (let i = targets.length - 1; i >= 0; i--) {
        const cg = targets[i];
        if (cg < currentFileStart && state.flatLines[cg]?.fileIdx === prevFileIdx) {
          setCursor(cg);
          return;
        }
      }
    }
  }
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
  const el = diffPane.querySelector('.cursor-line');
  if (el) el.scrollIntoView({ block: 'center' });
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

// ---------------------------------------------------------------------------
// Event handlers
// ---------------------------------------------------------------------------
document.addEventListener('keydown', (e) => {
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

    // Search
    case '/': e.preventDefault(); openSearch(); break;
    case 'n': e.preventDefault(); nextMatch(); break;
    case 'N': e.preventDefault(); prevMatch(); break;
    case 'Escape':
      if (state.searchActive) closeSearch();
      break;

    // Help
    case '?': e.preventDefault(); toggleHelp(); break;
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
// Utilities
// ---------------------------------------------------------------------------
function escapeHtml(str) {
  return str.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');
}

// ---------------------------------------------------------------------------
// Init
// ---------------------------------------------------------------------------
connect();
