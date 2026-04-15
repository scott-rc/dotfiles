import type { WebDiffFile, WebDiffLine } from "./types";
import { groupContextLines } from "./grouping";

export type DisplayItem =
  | { type: "file-header"; fileIdx: number; file: WebDiffFile }
  | { type: "hunk-sep"; fileIdx: number; hunkIdx: number }
  | { type: "line"; fileIdx: number; hunkIdx: number; line: WebDiffLine }
  | { type: "collapsed-context"; fileIdx: number; hunkIdx: number; groupKey: string; lines: WebDiffLine[]; count: number };

/** Heights in pixels for each item type. */
export const ITEM_HEIGHTS = {
  "file-header": 35,
  "hunk-sep": 20,
  line: 20,
  "collapsed-context": 28,
} as const;

/**
 * Flatten files into a single array of display items for virtual scrolling.
 * Context runs >3 lines are collapsed unless their groupKey is in expandedGroups.
 */
export function flattenFiles(files: WebDiffFile[], expandedGroups?: Set<string>): DisplayItem[] {
  const items: DisplayItem[] = [];
  for (let fi = 0; fi < files.length; fi++) {
    const file = files[fi];
    items.push({ type: "file-header", fileIdx: fi, file });
    for (let hi = 0; hi < file.hunks.length; hi++) {
      if (hi > 0) {
        items.push({ type: "hunk-sep", fileIdx: fi, hunkIdx: hi });
      }
      const grouped = groupContextLines(file.hunks[hi].lines);
      let contextGroupIdx = 0;
      for (const g of grouped) {
        if (g.type === "line") {
          items.push({ type: "line", fileIdx: fi, hunkIdx: hi, line: g.line });
        } else {
          const groupKey = `${fi}-${hi}-${contextGroupIdx}`;
          contextGroupIdx++;
          if (expandedGroups?.has(groupKey)) {
            for (const line of g.lines) {
              items.push({ type: "line", fileIdx: fi, hunkIdx: hi, line });
            }
          } else {
            items.push({
              type: "collapsed-context",
              fileIdx: fi,
              hunkIdx: hi,
              groupKey,
              lines: g.lines,
              count: g.count,
            });
          }
        }
      }
    }
  }
  return items;
}
