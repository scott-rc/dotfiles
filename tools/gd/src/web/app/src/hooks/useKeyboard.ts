import { useEffect, useCallback } from "preact/hooks";
import { mapKey, type Action } from "../utils/keyboard";
import type { DisplayItem } from "../utils/display";
import {
  cursor,
  displayItems,
  selectionAnchor,
  files,
  viewScope,
  singleFileIdx,
  fullContext,
  collapsedFiles,
  helpOpen,
  treeVisible,
  treeFocused,
  searchOpen,
  searchMatches,
  searchCurrentIdx,
  theme,
  sendMessage,
} from "../state/store";
import { jumpToCurrentMatch } from "../components/CommandPalette";

/** Check if display item at index is an added or deleted line. */
function isChangeLine(items: DisplayItem[], idx: number): boolean {
  const item = items[idx];
  if (!item || item.type !== "line") return false;
  return item.line.kind === "added" || item.line.kind === "deleted";
}

/** Find the next line-type item index at or after `from` in the given direction. */
function findContentLine(from: number, direction: 1 | -1): number {
  const items = displayItems.value;
  let i = from;
  while (i >= 0 && i < items.length) {
    if (items[i].type === "line") return i;
    i += direction;
  }
  return cursor.value; // no valid target, stay put
}

/** Get the file path for the display item at the given index. */
function filePathAt(idx: number): string | null {
  const item = displayItems.value[idx];
  if (!item) return null;
  const fileIdx = item.type === "file-header" ? item.fileIdx : item.fileIdx;
  return files.value[fileIdx]?.path ?? null;
}

export function useKeyboard(
  scrollToIndex: (idx: number, options?: { align: string }) => void,
  getVisibleRange: () => { startIndex: number; endIndex: number },
) {
  const dispatch = useCallback(
    (action: Action) => {
      const items = displayItems.value;
      if (items.length === 0) return;

      switch (action) {
        case "cursor-down": {
          const next = findContentLine(cursor.value + 1, 1);
          cursor.value = next;
          scrollToIndex(next, { align: "auto" });
          break;
        }
        case "cursor-up": {
          const prev = findContentLine(cursor.value - 1, -1);
          cursor.value = prev;
          scrollToIndex(prev, { align: "auto" });
          break;
        }
        case "half-page-down": {
          const range = getVisibleRange();
          const pageSize = Math.max(1, Math.floor((range.endIndex - range.startIndex) / 2));
          const target = Math.min(cursor.value + pageSize, items.length - 1);
          const next = findContentLine(target, 1);
          cursor.value = next;
          scrollToIndex(next, { align: "auto" });
          break;
        }
        case "half-page-up": {
          const range = getVisibleRange();
          const pageSize = Math.max(1, Math.floor((range.endIndex - range.startIndex) / 2));
          const target = Math.max(cursor.value - pageSize, 0);
          const prev = findContentLine(target, -1);
          cursor.value = prev;
          scrollToIndex(prev, { align: "auto" });
          break;
        }
        case "top": {
          const first = findContentLine(0, 1);
          cursor.value = first;
          scrollToIndex(0, { align: "start" });
          break;
        }
        case "bottom": {
          const last = findContentLine(items.length - 1, -1);
          cursor.value = last;
          scrollToIndex(last, { align: "end" });
          break;
        }
        case "center-cursor": {
          scrollToIndex(cursor.value, { align: "center" });
          break;
        }
        case "next-hunk": {
          // Jump to the start of the next change group (block of added/deleted lines)
          // Skip past current change group (if in one)
          let i = cursor.value;
          while (i < items.length && isChangeLine(items, i)) i++;
          // Find next change line
          while (i < items.length && !isChangeLine(items, i)) i++;
          if (i < items.length) {
            cursor.value = i;
            scrollToIndex(i, { align: "center" });
          }
          break;
        }
        case "prev-hunk": {
          // Jump to the start of the previous change group
          // Find start of current change group (if cursor is in one)
          let groupStart = cursor.value;
          while (groupStart > 0 && isChangeLine(items, groupStart - 1)) groupStart--;
          // Search backward from before current group for end of previous group
          let i = groupStart - 1;
          while (i >= 0 && !isChangeLine(items, i)) i--;
          if (i < 0) break; // Already at first group
          // Find start of that group
          while (i > 0 && isChangeLine(items, i - 1)) i--;
          cursor.value = i;
          scrollToIndex(i, { align: "center" });
          break;
        }
        case "next-file": {
          for (let i = cursor.value + 1; i < items.length; i++) {
            if (items[i].type === "file-header") {
              const line = findContentLine(i + 1, 1);
              cursor.value = line;
              scrollToIndex(i, { align: "start" });
              return;
            }
          }
          break;
        }
        case "prev-file": {
          // Find the file header for the current position, then go to the one before
          let currentFileHeader = -1;
          for (let i = cursor.value; i >= 0; i--) {
            if (items[i].type === "file-header") {
              currentFileHeader = i;
              break;
            }
          }
          for (let i = currentFileHeader - 1; i >= 0; i--) {
            if (items[i].type === "file-header") {
              const line = findContentLine(i + 1, 1);
              cursor.value = line;
              scrollToIndex(i, { align: "start" });
              return;
            }
          }
          break;
        }
        case "visual-select": {
          if (selectionAnchor.value !== null) {
            selectionAnchor.value = null;
          } else {
            selectionAnchor.value = cursor.value;
          }
          break;
        }
        case "yank": {
          if (selectionAnchor.value === null) return;
          const start = Math.min(selectionAnchor.value, cursor.value);
          const end = Math.max(selectionAnchor.value, cursor.value);
          const path = filePathAt(start);
          if (path) {
            // Find line numbers
            const startItem = items[start];
            const endItem = items[end];
            const startLine =
              startItem.type === "line"
                ? (startItem.line.new_lineno ?? startItem.line.old_lineno ?? 0)
                : 0;
            const endLine =
              endItem.type === "line"
                ? (endItem.line.new_lineno ?? endItem.line.old_lineno ?? 0)
                : 0;
            const text =
              startLine === endLine
                ? `${path}:${startLine}`
                : `${path}:${startLine}-${endLine}`;
            navigator.clipboard.writeText(text).catch(() => {});
          }
          selectionAnchor.value = null;
          break;
        }
        case "copy-path": {
          const path = filePathAt(cursor.value);
          if (path) navigator.clipboard.writeText(path).catch(() => {});
          break;
        }
        case "copy-abs-path": {
          const path = filePathAt(cursor.value);
          if (path) {
            // Best effort — we don't have the repo root, so just copy the path as-is
            navigator.clipboard.writeText(path).catch(() => {});
          }
          break;
        }
        case "toggle-single-file": {
          if (viewScope.value === "all") {
            // Find which file the cursor is in
            const item = items[cursor.value];
            if (item) singleFileIdx.value = item.fileIdx;
            viewScope.value = "single";
          } else {
            viewScope.value = "all";
          }
          break;
        }
        case "toggle-full-context": {
          fullContext.value = !fullContext.value;
          sendMessage({ type: "SetFullContext", enabled: fullContext.value });
          break;
        }
        case "toggle-collapse": {
          // Toggle collapse on the file containing cursor
          const item = items[cursor.value];
          if (!item) break;
          const fi = item.fileIdx;
          const next = new Set(collapsedFiles.value);
          if (next.has(fi)) {
            next.delete(fi);
          } else {
            next.add(fi);
          }
          collapsedFiles.value = next;
          break;
        }
        case "toggle-tree": {
          treeVisible.value = !treeVisible.value;
          if (!treeVisible.value) treeFocused.value = false;
          break;
        }
        case "toggle-tree-focus": {
          if (treeVisible.value) {
            treeFocused.value = !treeFocused.value;
          }
          break;
        }
        case "toggle-help": {
          helpOpen.value = !helpOpen.value;
          break;
        }
        case "open-search": {
          searchOpen.value = true;
          break;
        }
        case "next-match": {
          const matches = searchMatches.value;
          if (matches.length > 0) {
            searchCurrentIdx.value = (searchCurrentIdx.value + 1) % matches.length;
            jumpToCurrentMatch(scrollToIndex);
          }
          break;
        }
        case "prev-match": {
          const matches = searchMatches.value;
          if (matches.length > 0) {
            searchCurrentIdx.value =
              (searchCurrentIdx.value - 1 + matches.length) % matches.length;
            jumpToCurrentMatch(scrollToIndex);
          }
          break;
        }
        case "cancel-selection": {
          selectionAnchor.value = null;
          break;
        }
        case "cycle-theme": {
          const order: Array<"system" | "light" | "dark"> = ["system", "light", "dark"];
          const current = order.indexOf(theme.value);
          const next = order[(current + 1) % order.length];
          theme.value = next;
          localStorage.setItem("gd-theme", next);
          // Enable smooth transition, apply theme, then remove transition class
          document.documentElement.setAttribute("data-theme-transitioning", "");
          if (next === "system") {
            document.documentElement.removeAttribute("data-theme");
          } else {
            document.documentElement.setAttribute("data-theme", next);
          }
          setTimeout(() => {
            document.documentElement.removeAttribute("data-theme-transitioning");
          }, 200);
          break;
        }
        default:
          break;
      }
    },
    [scrollToIndex, getVisibleRange],
  );

  useEffect(() => {
    function handleKeyDown(e: KeyboardEvent) {
      // Don't intercept keys when search palette is open (it handles its own keys)
      if (searchOpen.value && e.key !== "Escape") return;
      // Don't intercept when help is open (only ? or Escape to close)
      if (helpOpen.value) {
        if (e.key === "?" || e.key === "Escape") {
          e.preventDefault();
          helpOpen.value = false;
        }
        return;
      }

      const action = mapKey(e);
      if (action) {
        e.preventDefault();
        dispatch(action);
      }
    }

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [dispatch]);

  return dispatch;
}
