import { signal, computed } from "@preact/signals";
import type { WebDiffFile, WebTreeEntry } from "../utils/types";
import { flattenFiles } from "../utils/display";

// Data from server
export const files = signal<WebDiffFile[]>([]);
export const tree = signal<WebTreeEntry[]>([]);
export const branch = signal("");
export const sourceLabel = signal("");

// Connection state
export const connected = signal(false);

// View state
export const viewScope = signal<"all" | "single">("all");
export const singleFileIdx = signal(0);
export const fullContext = signal(false);

// Collapse state
export const collapsedFiles = signal<Set<number>>(new Set());

// Cursor state
export const cursor = signal(0);

// Visual selection state
export const selectionAnchor = signal<number | null>(null);

// Tree state
export const treeVisible = signal(true);
export const treeFocused = signal(false);
export const treeWidth = signal(
  parseInt(localStorage.getItem("gd-tree-width") ?? "220", 10),
);

// Search state
export const searchOpen = signal(false);
export const searchQuery = signal("");
export const searchMatches = signal<number[]>([]);
export const searchCurrentIdx = signal(0);

// Theme state
export const theme = signal<"system" | "light" | "dark">(
  (localStorage.getItem("gd-theme") as "system" | "light" | "dark") ?? "system",
);

// UI state
export const helpOpen = signal(false);

// Derived: flat display items (respects view scope and collapsed files)
export const displayItems = computed(() => {
  const allItems = flattenFiles(files.value);
  const collapsed = collapsedFiles.value;
  const scope = viewScope.value;
  const fileIdx = singleFileIdx.value;

  return allItems.filter((item) => {
    // Single-file filter
    if (scope === "single" && item.fileIdx !== fileIdx) return false;
    // Collapsed file filter: show header, hide content
    if (collapsed.has(item.fileIdx) && item.type !== "file-header") return false;
    return true;
  });
});

// WebSocket reference for sending messages
let wsRef: WebSocket | null = null;
export function setWs(ws: WebSocket | null) {
  wsRef = ws;
}
export function sendMessage(msg: object) {
  if (wsRef?.readyState === WebSocket.OPEN) {
    wsRef.send(JSON.stringify(msg));
  }
}

// Expose state for E2E tests
Object.defineProperty(window, "__gdState", {
  get() {
    const items = displayItems.value;
    // Compute change group starts: indices where a run of added/deleted lines begins
    const changeGroupStarts: number[] = [];
    for (let i = 0; i < items.length; i++) {
      const item = items[i];
      if (item.type !== "line") continue;
      const kind = item.line.kind;
      if (kind !== "added" && kind !== "deleted") continue;
      // Check if previous line-type item was not a change
      let prevIsChange = false;
      for (let j = i - 1; j >= 0; j--) {
        const prev = items[j];
        if (prev.type === "line") {
          prevIsChange = prev.line.kind === "added" || prev.line.kind === "deleted";
        }
        break; // any non-consecutive-line breaks the group
      }
      if (!prevIsChange) {
        changeGroupStarts.push(i);
      }
    }

    return {
      cursorLine: cursor.value,
      fullContext: fullContext.value,
      flatLines: items.map((item) => {
        const base = { type: item.type, fileIdx: item.fileIdx };
        if (item.type === "line") return { ...base, data: { kind: item.line.kind } };
        return base;
      }),
      changeGroupStarts,
    };
  },
});
