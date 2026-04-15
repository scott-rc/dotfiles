import { useRef, useEffect } from "preact/hooks";
import {
  searchOpen,
  searchQuery,
  searchMatches,
  searchCurrentIdx,
  displayItems,
  cursor,
} from "../state/store";
import { getGlobalScrollToIndex } from "./DiffPane";

function performSearch(query: string) {
  if (!query) {
    searchMatches.value = [];
    searchCurrentIdx.value = 0;
    return;
  }
  const lower = query.toLowerCase();
  const items = displayItems.value;
  const matches: number[] = [];
  for (let i = 0; i < items.length; i++) {
    const item = items[i];
    if (item.type === "line" && item.line.raw_content.toLowerCase().includes(lower)) {
      matches.push(i);
    }
  }
  searchMatches.value = matches;
  searchCurrentIdx.value = 0;
}

export function jumpToCurrentMatch(
  scrollToIndex?: (idx: number, opts?: { align: string }) => void,
) {
  const matches = searchMatches.value;
  if (matches.length === 0) return;
  const idx = searchCurrentIdx.value;
  const target = matches[idx];
  cursor.value = target;
  const scroll = scrollToIndex ?? getGlobalScrollToIndex();
  scroll?.(target, { align: "center" });
}

export function CommandPalette() {
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (searchOpen.value) {
      inputRef.current?.focus();
    }
  }, [searchOpen.value]);

  if (!searchOpen.value) return null;

  const matchCount = searchMatches.value.length;
  const currentIdx = searchCurrentIdx.value;

  return (
    <div id="search-bar" class="fixed inset-0 z-50 flex items-start justify-center pt-[20vh]" role="dialog" aria-label="Search diff content" aria-modal="true">
      <div
        class="absolute inset-0 bg-black/50"
        onClick={() => {
          searchOpen.value = false;
        }}
      />
      <div class="relative bg-[var(--bg-secondary)] border border-[var(--fg-sep)] rounded-lg shadow-xl w-[500px] max-w-[90vw] animate-in">
        <div class="flex items-center gap-2 px-4 py-3">
          <span class="text-[var(--fg-gutter)] text-sm">/</span>
          <input
            ref={inputRef}
            id="search-input"
            type="text"
            value={searchQuery.value}
            class="flex-1 bg-transparent outline-none text-sm text-[var(--fg)]"
            placeholder="Search diff content..."
            aria-label="Search query"
            onInput={(e) => {
              const q = (e.target as HTMLInputElement).value;
              searchQuery.value = q;
              performSearch(q);
            }}
            onKeyDown={(e) => {
              if (e.key === "Escape") {
                e.stopPropagation();
                searchOpen.value = false;
              } else if (e.key === "Enter") {
                jumpToCurrentMatch();
              } else if (e.key === "ArrowDown") {
                e.preventDefault();
                if (matchCount > 0) {
                  searchCurrentIdx.value = (currentIdx + 1) % matchCount;
                  jumpToCurrentMatch();
                }
              } else if (e.key === "ArrowUp") {
                e.preventDefault();
                if (matchCount > 0) {
                  searchCurrentIdx.value = (currentIdx - 1 + matchCount) % matchCount;
                  jumpToCurrentMatch();
                }
              }
            }}
          />
          {matchCount > 0 && (
            <span id="search-count" class="text-xs text-[var(--fg-gutter)]">
              {currentIdx + 1}/{matchCount}
            </span>
          )}
          {searchQuery.value && matchCount === 0 && (
            <span class="text-xs text-[var(--fg-deleted-marker)]">No matches</span>
          )}
        </div>
      </div>
    </div>
  );
}
