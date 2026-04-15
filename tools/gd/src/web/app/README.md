# gd Web UI

Preact + TypeScript + Tailwind CSS frontend for the `gd --web` browser-based diff viewer. Built with Vite, embedded into the Rust binary via `rust-embed` at compile time.

## Architecture

### Stack

| Layer | Choice | Why |
|-------|--------|-----|
| Framework | Preact + `@preact/signals` | Fine-grained reactivity, ~4KB bundle, signals avoid re-render cascades |
| Styling | Tailwind CSS v4 | Utility-first, rapid iteration, no CSS files to manage |
| Components | Base UI (`@base-ui/react`) | Headless accessible primitives (Collapsible, Dialog) via `preact/compat` |
| Virtual scroll | `@tanstack/react-virtual` | Keeps DOM < 2000 nodes on 10K+ line diffs, via `preact/compat` |
| Build | Vite (via VitePlus `vp` CLI) | HMR in dev, optimized production build |
| Tests | Vitest (unit), Playwright (E2E) | Unit tests for pure functions, E2E for user-facing behavior |

### Data Flow

```
Rust server (axum)
    │
    ▼ WebSocket (JSON)
useWebSocket hook
    │
    ▼ parses DiffData message
Signals store (store.ts)
    │
    ▼ computed signal
displayItems (flat list for virtualizer)
    │
    ▼ @tanstack/react-virtual
DiffPane renders only visible items
```

1. Rust server pushes `DiffData` messages over WebSocket containing `files`, `tree`, `branch`, `source_label`
2. `useWebSocket` hook parses messages and writes to signals in `store.ts`
3. `displayItems` computed signal flattens files into a single array, applying context grouping, file collapse, and single-file filtering
4. `@tanstack/react-virtual` virtualizes the flat list -- only visible items plus buffer exist in the DOM
5. User actions (keyboard, clicks) update signals, which re-derive `displayItems` and trigger minimal re-renders

### State Management

All state lives in `state/store.ts` as top-level `@preact/signals` signals. No context providers, no reducers -- signals are imported directly by components that need them.

**Server data signals** -- populated from WebSocket `DiffData` messages:
- `files`, `tree`, `branch`, `sourceLabel`

**View state signals** -- user-controlled UI state:
- `cursor`, `viewScope` (`'all'` | `'single'`), `singleFileIdx`, `fullContext`
- `collapsedFiles` (Set of file indices), `expandedContextGroups` (Set of group keys)
- `selectionAnchor` (visual selection start, null when inactive)

**Tree signals**: `treeVisible`, `treeFocused`, `treeWidth`

**Search signals**: `searchOpen`, `searchQuery`, `searchMatches`, `searchCurrentIdx`

**UI signals**: `helpOpen`, `theme` (`'system'` | `'light'` | `'dark'`)

**Key derived signal**: `displayItems` -- computed from `files` + `expandedContextGroups` + `collapsedFiles` + `viewScope` + `singleFileIdx`. This is the single flat array that the virtualizer consumes. All filtering (single-file, collapsed files) and grouping (context collapse) happens here.

**Persistence**: `theme` and `treeWidth` are persisted to localStorage. Read on init, written on change.

### Display Model

The virtualizer needs a flat list of items with known heights. `display.ts` defines `DisplayItem`:

| Type | Height | Description |
|------|--------|-------------|
| `file-header` | 35px | File path, chevron, +N -N stats |
| `hunk-sep` | 20px | Visual separator between hunks |
| `line` | 20px | Diff line (context, added, or deleted) |
| `collapsed-context` | 28px | "N unmodified lines" placeholder |

`flattenFiles()` converts `WebDiffFile[]` into `DisplayItem[]`. Context runs > 3 consecutive context lines are collapsed unless their group key is in `expandedContextGroups`. Group keys are deterministic: `${fileIdx}-${hunkIdx}-${contextGroupIdx}`.

### Component Hierarchy

```
App
├── Header (fixed top bar: branch, source label, theme toggle, help button)
├── main container (flex row)
│   ├── DiffPane (virtual scroll container, takes remaining width)
│   │   └── per virtual item:
│   │       ├── file-header → DiffFile header row
│   │       ├── hunk-sep → dashed separator
│   │       ├── line → DiffLine (gutter + border + content)
│   │       └── collapsed-context → CollapsedContext ("N unmodified lines")
│   └── Tree (right sidebar, resizable via drag handle)
│       └── TreeEntry rows (files and directories with icons)
├── CommandPalette (modal, opened by `/` or `Cmd+K`)
└── HelpOverlay (modal, opened by `?`)
```

### Keyboard Architecture

Keys are mapped in `utils/keyboard.ts` -- a flat mapping from key strings to action identifiers. No modes, no context-dependent behavior (except when search input or command palette has focus).

`hooks/useKeyboard.ts` attaches a global keydown listener that:
1. Checks if command palette or help overlay is open (they intercept keys)
2. Maps the key to an action via `keyboard.ts`
3. Dispatches the action (updates signals, scrolls virtualizer, sends WebSocket messages)

All vim keys have standard equivalents (arrows, Home/End, Page Up/Down).

### Visual Design Decisions

- **Single line number gutter**: Shows `old_lineno` for deletes, `new_lineno` for context/adds. Compact, reduces noise vs dual gutters.
- **3px left border**: Green for added, red for deleted, none for context. Replaces `+`/`-` text markers -- also works for colorblind users (positional, not just color).
- **Word-level highlights**: `<mark>` tags from Rust HTML rendering get brighter background styling to pinpoint exact changes within a line.
- **No bottom status bar**: Scroll position is visible from the scrollbar; branch/source info lives in the header.
- **GitHub Dark palette**: Consistent with the terminal pager's color scheme.

### Protocol (WebSocket Messages)

Types mirror Rust `src/web/protocol.rs`. Defined in `utils/types.ts`.

**Server → Client**:
- `DiffData` -- `{ type: "DiffData", files: WebDiffFile[], tree: WebTreeEntry[], branch: string, source_label: string }`

**Client → Server**:
- `SetFullContext` -- `{ type: "SetFullContext", enabled: boolean }` (sent when user presses `o`)

**Types**:
- `WebDiffFile` -- `{ path, old_path, status, hunks[] }`
- `WebDiffHunk` -- `{ old_start, new_start, lines[] }`
- `WebDiffLine` -- `{ kind, content_html, raw_content, old_lineno, new_lineno, line_idx }`
- `WebTreeEntry` -- `{ label, depth, file_idx, status, is_dir, collapsed, icon, icon_color }`

Content is pre-rendered as HTML on the Rust side (`content_html` has `<span>` tags for syntax highlighting, `<mark>` tags for word-level diffs). The frontend renders it with `dangerouslySetInnerHTML`.

### Navigation Logic

Pure functions in `utils/navigation.ts` handle cursor movement through the display items list:

- `findContentLine()` -- find the next line-type item in a direction (skips headers/separators)
- `findNextHunk()` / `findPrevHunk()` -- jump between change groups (runs of added/deleted lines)
- `findNextFile()` / `findPrevFile()` -- jump between file headers
- `isChangeLine()` -- check if a display item is an added or deleted line

These are pure functions taking `DisplayItem[]` and an index, returning an index. Tested in `__tests__/navigation.test.ts`.

## Project Structure

```
src/
├── main.tsx                    # Entry point, renders <App> into #app
├── index.css                   # Tailwind imports + custom properties
├── state/
│   └── store.ts                # All signals + displayItems computed + WebSocket ref
├── utils/
│   ├── types.ts                # TypeScript types matching Rust protocol
│   ├── display.ts              # DisplayItem type + flattenFiles() for virtualizer
│   ├── grouping.ts             # Context line grouping (consecutive context > 3 → collapsed)
│   ├── keyboard.ts             # Key-to-action mapping
│   ├── navigation.ts           # Pure cursor navigation functions
│   └── perf.ts                 # window.__gdPerf performance metrics
├── hooks/
│   ├── useWebSocket.ts         # WebSocket connection with auto-reconnect
│   └── useKeyboard.ts          # Global keyboard handler, dispatches actions
├── components/
│   ├── App.tsx                 # Root layout (header + diff pane + tree + modals)
│   ├── DiffPane.tsx            # Virtual-scrolled diff container with cursor/selection/search highlighting
│   ├── DiffFile.tsx            # File header row (path, chevron, stats)
│   ├── DiffLine.tsx            # Single diff line (gutter, border, content)
│   ├── CollapsedContext.tsx     # "N unmodified lines" placeholder
│   ├── Tree.tsx                # File tree sidebar with drag-to-resize
│   ├── CommandPalette.tsx      # Modal search with real-time match highlighting
│   └── HelpOverlay.tsx         # Keybinding reference overlay
└── __tests__/
    ├── keyboard.test.ts        # Key mapping coverage
    ├── grouping.test.ts        # Context grouping edge cases
    ├── display.test.ts         # flattenFiles() behavior
    ├── navigation.test.ts      # Cursor navigation functions
    ├── types.test.ts           # Type guard tests
    └── placeholder.test.ts     # Vitest smoke test
```

## Development

### Dev Workflow

Run two terminals:

```bash
# Terminal 1: Rust server on fixed port
cd tools/gd && cargo run --features web -- --web --port 3845 --no-open

# Terminal 2: Vite dev server with WebSocket proxy
cd tools/gd/src/web/app && pnpm dev
```

Vite dev server runs on `:5173` and proxies `/ws` to `ws://localhost:3845/ws`. Frontend changes hot-reload instantly; only protocol/server changes need a Rust rebuild.

### Build

```bash
pnpm build     # production build → dist/
pnpm test      # vitest unit tests
pnpm check     # typecheck
pnpm lint      # oxlint
```

All commands use VitePlus (`vp`) under the hood. The `dist/` directory is embedded into the Rust binary by `rust-embed`. `build.rs` in the Rust crate triggers `vp build` automatically when frontend sources change (mtime comparison against `dist/index.html`).

### E2E Tests

E2E tests live in `../e2e/` (relative to this app, at `tools/gd/src/web/e2e/`). They use Playwright against the full Rust server + embedded frontend.

```bash
cd tools/gd/src/web/e2e
./fixtures/setup.sh            # create test repo with known diff state
npm test                       # run all tests
npm run test:headed            # run with visible browser
```

### Performance Metrics

The web UI exposes `window.__gdPerf` for runtime performance measurement:

| Metric | Target | Access |
|--------|--------|--------|
| Initial load | < 100ms | `__gdPerf.getMetrics().initialLoad.totalMs` |
| Render time | < 20ms | `__gdPerf.getMetrics().render.lastMs` |
| Navigation latency | < 10ms | `__gdPerf.getMetrics().navigation` |
| DOM node count | < 2000 | `__gdPerf.getMetrics().dom.nodeCount` |

## Out of Scope

These features are intentionally excluded from the web UI:

- **Staging/unstaging** -- stays in the TUI pager only
- **Side-by-side diff view** -- unified view only
- **Mobile/responsive layout** -- desktop only
- **Branch switching** -- view-only, shows current branch in header
- **Configurable tree position** -- fixed right side
