import { useCallback, useEffect } from "preact/hooks";
import { useWebSocket } from "../hooks/useWebSocket";
import {
  connected,
  files,
  branch,
  sourceLabel,
  treeVisible,
  cursor,
  displayItems,
  theme,
} from "../state/store";
import { DiffPane } from "./DiffPane";
import { Tree } from "./Tree";
import { CommandPalette } from "./CommandPalette";
import { HelpOverlay } from "./HelpOverlay";

export function App() {
  useWebSocket();

  // Apply theme on mount
  useEffect(() => {
    const t = theme.value;
    if (t === "system") {
      document.documentElement.removeAttribute("data-theme");
    } else {
      document.documentElement.setAttribute("data-theme", t);
    }
  }, []);

  const handleFileClick = useCallback((fileIdx: number) => {
    const items = displayItems.value;
    for (let i = 0; i < items.length; i++) {
      if (items[i].type === "file-header" && items[i].fileIdx === fileIdx) {
        for (let j = i + 1; j < items.length; j++) {
          if (items[j].type === "line") {
            cursor.value = j;
            break;
          }
        }
        break;
      }
    }
  }, []);

  return (
    <div class="h-screen flex flex-col">
      <header class="flex items-center gap-3 px-4 py-2 bg-[var(--bg-secondary)] border-b border-[var(--fg-sep)] shrink-0 shadow-sm" role="banner" aria-label="Diff viewer header">
        <h1 class="text-sm font-bold">gd</h1>
        {branch.value && (
          <span class="text-sm text-[var(--fg-file-header)]">
            {branch.value}
          </span>
        )}
        {sourceLabel.value && (
          <span class="text-xs text-[var(--fg-gutter)]">
            {sourceLabel.value}
          </span>
        )}
        <span class="ml-auto text-xs text-[var(--fg-gutter)]">
          {files.value.length} files
        </span>
        <span
          class={`text-xs px-1.5 py-0.5 rounded ${connected.value ? "bg-green-900/50 text-green-400" : "bg-red-900/50 text-red-400"}`}
          role="status"
          aria-live="polite"
        >
          {connected.value ? "Connected" : "Disconnected"}
        </span>
      </header>
      <div class="flex flex-1 min-h-0">
        <DiffPane />
        {treeVisible.value && <Tree onFileClick={handleFileClick} />}
      </div>
      <CommandPalette />
      <HelpOverlay />
    </div>
  );
}
