import { useRef, useCallback } from "preact/hooks";
import { tree, treeWidth, treeFocused, displayItems, cursor } from "../state/store";
import type { WebTreeEntry } from "../utils/types";

interface TreeProps {
  onFileClick: (fileIdx: number) => void;
}

function TreeEntryRow({
  entry,
  i,
  isActive,
  isFocused,
  onClick,
}: {
  entry: WebTreeEntry;
  i: number;
  isActive: boolean;
  isFocused: boolean;
  onClick: () => void;
}) {
  const cls = `tree-entry${isActive ? " active" : ""}${entry.is_dir ? " dir" : ""}${!isFocused ? " unfocused" : ""} flex items-center gap-1 px-2 py-0.5 text-xs cursor-pointer truncate select-none transition-colors duration-100
        ${isActive ? "bg-[var(--bg-cursor)] text-white" : "hover:bg-[var(--bg-secondary)]"}`;
  return (
    <div
      class={cls}
      style={{ paddingLeft: `${entry.depth * 12 + 8}px` }}
      data-tree-idx={i}
      onClick={onClick}
    >
      {entry.is_dir && (
        <span class="text-[var(--fg-gutter)] text-[10px] w-3">
          {entry.collapsed ? "▸" : "▾"}
        </span>
      )}
      <span
        class="tree-icon shrink-0"
        style={{
          color: entry.icon_color || "var(--fg-gutter)",
          fontFamily: "var(--font-icons)",
        }}
      >
        {entry.icon}
      </span>
      <span class="tree-label truncate">{entry.label}</span>
      {entry.file_idx !== null && entry.status && (
        <span class="ml-auto shrink-0 text-[10px] text-[var(--fg-gutter)]">
          {getEntryStats(entry)}
        </span>
      )}
    </div>
  );
}

function getEntryStats(_entry: WebTreeEntry): string {
  // Stats are not in the tree protocol — we'd need to compute from files.
  // For now, show nothing (stats are visible in file headers already).
  return "";
}

export function Tree({ onFileClick }: TreeProps) {
  const dragRef = useRef<{ startX: number; startWidth: number } | null>(null);
  const width = treeWidth.value;
  const focused = treeFocused.value;

  // Find the active file (the file the cursor is in)
  const activeFileIdx = (() => {
    const items = displayItems.value;
    const item = items[cursor.value];
    return item?.fileIdx ?? null;
  })();

  const handleMouseDown = useCallback((e: MouseEvent) => {
    e.preventDefault();
    dragRef.current = { startX: e.clientX, startWidth: treeWidth.value };

    function handleMouseMove(e: MouseEvent) {
      if (!dragRef.current) return;
      const delta = dragRef.current.startX - e.clientX;
      const newWidth = Math.max(120, Math.min(500, dragRef.current.startWidth + delta));
      treeWidth.value = newWidth;
    }

    function handleMouseUp() {
      if (dragRef.current) {
        localStorage.setItem("gd-tree-width", String(treeWidth.value));
      }
      dragRef.current = null;
      window.removeEventListener("mousemove", handleMouseMove);
      window.removeEventListener("mouseup", handleMouseUp);
    }

    window.addEventListener("mousemove", handleMouseMove);
    window.addEventListener("mouseup", handleMouseUp);
  }, []);

  return (
    <nav
      id="tree"
      class={`flex shrink-0 border-l border-[var(--fg-sep)] bg-[var(--bg)] ${focused ? "" : "opacity-80"}`}
      style={{ width: `${width}px` }}
      aria-label="File tree"
    >
      <div
        class="w-1 cursor-col-resize hover:bg-[var(--fg-sep)] transition-colors"
        onMouseDown={handleMouseDown}
        role="separator"
        aria-orientation="vertical"
        aria-label="Resize tree panel"
      />
      <div class="flex-1 overflow-y-auto py-1">
        {tree.value.map((entry, i) => (
          <TreeEntryRow
            key={`${entry.label}-${entry.depth}-${i}`}
            entry={entry}
            i={i}
            isActive={entry.file_idx === activeFileIdx}
            isFocused={focused}
            onClick={() => {
              if (entry.file_idx !== null) {
                onFileClick(entry.file_idx);
              }
            }}
          />
        ))}
      </div>
    </nav>
  );
}
