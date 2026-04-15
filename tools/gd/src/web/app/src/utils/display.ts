import type { WebDiffFile, WebDiffLine } from "./types";

export type DisplayItem =
  | { type: "file-header"; fileIdx: number; file: WebDiffFile }
  | { type: "hunk-sep"; fileIdx: number; hunkIdx: number }
  | { type: "line"; fileIdx: number; hunkIdx: number; line: WebDiffLine };

/** Heights in pixels for each item type. */
export const ITEM_HEIGHTS = {
  "file-header": 35,
  "hunk-sep": 20,
  line: 20,
} as const;

/** Flatten files into a single array of display items for virtual scrolling. */
export function flattenFiles(files: WebDiffFile[]): DisplayItem[] {
  const items: DisplayItem[] = [];
  for (let fi = 0; fi < files.length; fi++) {
    const file = files[fi];
    items.push({ type: "file-header", fileIdx: fi, file });
    for (let hi = 0; hi < file.hunks.length; hi++) {
      if (hi > 0) {
        items.push({ type: "hunk-sep", fileIdx: fi, hunkIdx: hi });
      }
      for (const line of file.hunks[hi].lines) {
        items.push({ type: "line", fileIdx: fi, hunkIdx: hi, line });
      }
    }
  }
  return items;
}
