import { useRef, useCallback, useEffect } from "preact/hooks";
import { useVirtualizer } from "@tanstack/react-virtual";
import { displayItems, cursor, selectionAnchor, collapsedFiles, searchMatches, searchCurrentIdx } from "../state/store";
import { ITEM_HEIGHTS, type DisplayItem } from "../utils/display";
import { DiffLine } from "./DiffLine";
import { CollapsedContext } from "./CollapsedContext";
import { useKeyboard } from "../hooks/useKeyboard";

// Global scroll function for use by CommandPalette
let _globalScrollToIndex: ((idx: number, opts?: { align: string }) => void) | null = null;
export function getGlobalScrollToIndex() {
  return _globalScrollToIndex;
}

function toggleFileCollapse(fileIdx: number) {
  const next = new Set(collapsedFiles.value);
  if (next.has(fileIdx)) {
    next.delete(fileIdx);
  } else {
    next.add(fileIdx);
  }
  collapsedFiles.value = next;
}

function FileHeader({ item }: { item: DisplayItem & { type: "file-header" } }) {
  const file = item.file;
  const isCollapsed = collapsedFiles.value.has(item.fileIdx);
  const additions = file.hunks.reduce(
    (sum, h) => sum + h.lines.filter((l) => l.kind === "added").length,
    0,
  );
  const deletions = file.hunks.reduce(
    (sum, h) => sum + h.lines.filter((l) => l.kind === "deleted").length,
    0,
  );

  return (
    <div
      class="file-header sticky top-0 z-10 flex items-center gap-2 px-3 bg-[var(--bg-secondary)] border-b border-[var(--fg-sep)] cursor-pointer select-none hover:brightness-110 transition-[filter] duration-100"
      style={{
        height: `${ITEM_HEIGHTS["file-header"]}px`,
        lineHeight: `${ITEM_HEIGHTS["file-header"]}px`,
      }}
      role="button"
      aria-expanded={!isCollapsed}
      aria-label={`${file.path} — click to ${isCollapsed ? "expand" : "collapse"}`}
      onClick={() => toggleFileCollapse(item.fileIdx)}
    >
      <span
        class="text-[var(--fg-gutter)] text-xs transition-transform duration-150"
        style={{ transform: isCollapsed ? "rotate(-90deg)" : "rotate(0deg)" }}
      >
        ▾
      </span>
      <span class="font-semibold text-sm text-[var(--fg-file-header)] truncate">
        {file.path}
      </span>
      <span class="ml-auto flex gap-2 text-xs shrink-0">
        {additions > 0 && (
          <span class="text-[var(--fg-added-marker)]">+{additions}</span>
        )}
        {deletions > 0 && (
          <span class="text-[var(--fg-deleted-marker)]">-{deletions}</span>
        )}
      </span>
    </div>
  );
}

function HunkSep() {
  return (
    <div
      class="hunk-sep flex items-center px-3 text-xs text-[var(--fg-sep)] select-none"
      style={{ height: `${ITEM_HEIGHTS["hunk-sep"]}px` }}
    >
      <span class="w-full border-t border-dashed border-[var(--fg-sep)]" />
    </div>
  );
}

function isInSelection(idx: number): boolean {
  const anchor = selectionAnchor.value;
  if (anchor === null) return false;
  const cur = cursor.value;
  const lo = Math.min(anchor, cur);
  const hi = Math.max(anchor, cur);
  return idx >= lo && idx <= hi;
}

function DisplayRow({
  item,
  index,
  isMatch,
  isCurrentMatch,
}: {
  item: DisplayItem;
  index: number;
  isMatch: boolean;
  isCurrentMatch: boolean;
}) {
  const isCursor = index === cursor.value && item.type === "line";
  const isSelected = isInSelection(index) && item.type === "line";

  // Build class list — cursor-line and visual-selected can coexist
  const classes: string[] = [];
  if (isCursor) classes.push("cursor-line");
  if (isSelected) classes.push("visual-selected");
  if (isCurrentMatch) classes.push("search-match");
  else if (isMatch) classes.push("search-match");

  const bgClass = isCurrentMatch
    ? "bg-[var(--bg-search-current,rgba(255,213,79,0.35))]"
    : isMatch
      ? "bg-[var(--bg-search-match,rgba(255,213,79,0.15))]"
      : isSelected
        ? "bg-[var(--bg-visual)]"
        : isCursor
          ? "bg-[var(--bg-cursor)]"
          : "";
  const extraClass = [...classes, bgClass].filter(Boolean).join(" ");

  switch (item.type) {
    case "file-header":
      return <FileHeader item={item} />;
    case "hunk-sep":
      return <HunkSep />;
    case "collapsed-context":
      return <CollapsedContext groupKey={item.groupKey} count={item.count} />;
    case "line": {
      const kindClass = item.line.kind === "added" ? " line-added" : item.line.kind === "deleted" ? " line-deleted" : "";
      return (
        <div class={`diff-line${kindClass} ${extraClass}`} data-flat-idx={index}>
          <DiffLine line={item.line} />
        </div>
      );
    }
  }
}

export function DiffPane() {
  const parentRef = useRef<HTMLDivElement>(null);
  const items = displayItems.value;

  const virtualizer = useVirtualizer({
    count: items.length,
    getScrollElement: () => parentRef.current,
    estimateSize: (i) => ITEM_HEIGHTS[items[i].type],
    overscan: 50,
  });

  const scrollToIndex = useCallback(
    (idx: number, options?: { align: string }) => {
      virtualizer.scrollToIndex(idx, {
        align: (options?.align as "auto" | "start" | "center" | "end") ?? "auto",
      });
    },
    [virtualizer],
  );

  const getVisibleRange = useCallback(() => {
    const vItems = virtualizer.getVirtualItems();
    if (vItems.length === 0) return { startIndex: 0, endIndex: 0 };
    return {
      startIndex: vItems[0].index,
      endIndex: vItems[vItems.length - 1].index,
    };
  }, [virtualizer]);

  useKeyboard(scrollToIndex, getVisibleRange);

  // Register global scroll function
  useEffect(() => {
    _globalScrollToIndex = scrollToIndex;
    return () => { _globalScrollToIndex = null; };
  }, [scrollToIndex]);

  // Determine which lines are search matches for highlighting
  const matchSet = new Set(searchMatches.value);
  const currentMatchIdx = searchMatches.value[searchCurrentIdx.value] ?? -1;

  return (
    <div ref={parentRef} id="diff-pane" class="flex-1 overflow-y-auto" tabIndex={0} role="main" aria-label="Diff content">
      {items.length === 0 ? (
        <p class="p-4 text-[var(--fg-gutter)] text-sm">No changes</p>
      ) : (
        <div
          style={{
            height: `${virtualizer.getTotalSize()}px`,
            width: "100%",
            position: "relative",
          }}
        >
          {virtualizer.getVirtualItems().map((vRow) => (
            <div
              key={vRow.index}
              style={{
                position: "absolute",
                top: 0,
                left: 0,
                width: "100%",
                height: `${vRow.size}px`,
                transform: `translateY(${vRow.start}px)`,
              }}
            >
              <DisplayRow
                item={items[vRow.index]}
                index={vRow.index}
                isMatch={matchSet.has(vRow.index)}
                isCurrentMatch={vRow.index === currentMatchIdx}
              />
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
