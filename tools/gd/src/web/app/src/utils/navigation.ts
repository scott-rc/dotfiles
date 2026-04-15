import type { DisplayItem } from "./display";

/** Check if display item at index is an added or deleted line. */
export function isChangeLine(items: DisplayItem[], idx: number): boolean {
  const item = items[idx];
  if (!item || item.type !== "line") return false;
  return item.line.kind === "added" || item.line.kind === "deleted";
}

/** Find the next line-type item index at or after `from` in the given direction. Returns `fallback` if none found. */
export function findContentLine(
  items: DisplayItem[],
  from: number,
  direction: 1 | -1,
  fallback: number,
): number {
  let i = from;
  while (i >= 0 && i < items.length) {
    if (items[i].type === "line") return i;
    i += direction;
  }
  return fallback;
}

/** Get the file path for the display item at the given index. */
export function filePathAt(
  items: DisplayItem[],
  filePaths: string[],
  idx: number,
): string | null {
  const item = items[idx];
  if (!item) return null;
  return filePaths[item.fileIdx] ?? null;
}

/** Find the index of the start of the next change group after `cursorIdx`. Returns null if already at last group. */
export function findNextHunk(items: DisplayItem[], cursorIdx: number): number | null {
  let i = cursorIdx;
  // Skip past current change group (if in one)
  while (i < items.length && isChangeLine(items, i)) i++;
  // Find next change line
  while (i < items.length && !isChangeLine(items, i)) i++;
  return i < items.length ? i : null;
}

/** Find the index of the start of the previous change group before `cursorIdx`. Returns null if already at first group. */
export function findPrevHunk(items: DisplayItem[], cursorIdx: number): number | null {
  // Find start of current change group (only if cursor is on a change line)
  let groupStart = cursorIdx;
  if (isChangeLine(items, cursorIdx)) {
    while (groupStart > 0 && isChangeLine(items, groupStart - 1)) groupStart--;
  }
  // Search backward from before current group for end of previous group
  let i = groupStart - 1;
  while (i >= 0 && !isChangeLine(items, i)) i--;
  if (i < 0) return null; // Already at first group
  // Find start of that group
  while (i > 0 && isChangeLine(items, i - 1)) i--;
  return i;
}

/** Find the first content line after the next file header. Returns { cursor, headerIdx } or null. */
export function findNextFile(
  items: DisplayItem[],
  cursorIdx: number,
  fallback: number,
): { cursor: number; headerIdx: number } | null {
  for (let i = cursorIdx + 1; i < items.length; i++) {
    if (items[i].type === "file-header") {
      const line = findContentLine(items, i + 1, 1, fallback);
      return { cursor: line, headerIdx: i };
    }
  }
  return null;
}

/** Find the first content line after the previous file header. Returns { cursor, headerIdx } or null. */
export function findPrevFile(
  items: DisplayItem[],
  cursorIdx: number,
  fallback: number,
): { cursor: number; headerIdx: number } | null {
  if (items.length === 0) return null;
  // Find the file header for the current position, then go to the one before
  let currentFileHeader = -1;
  for (let i = Math.min(cursorIdx, items.length - 1); i >= 0; i--) {
    if (items[i].type === "file-header") {
      currentFileHeader = i;
      break;
    }
  }
  for (let i = currentFileHeader - 1; i >= 0; i--) {
    if (items[i].type === "file-header") {
      const line = findContentLine(items, i + 1, 1, fallback);
      return { cursor: line, headerIdx: i };
    }
  }
  return null;
}
