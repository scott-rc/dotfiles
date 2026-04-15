import type { WebDiffLine } from "./types";

export type GroupedItem =
  | { type: "line"; line: WebDiffLine }
  | { type: "collapsed-context"; lines: WebDiffLine[]; count: number };

/** Group consecutive context lines that exceed `threshold` into collapsible items. */
export function groupContextLines(
  lines: WebDiffLine[],
  threshold = 3,
): GroupedItem[] {
  const result: GroupedItem[] = [];
  let i = 0;

  while (i < lines.length) {
    if (lines[i].kind === "context") {
      let j = i;
      while (j < lines.length && lines[j].kind === "context") j++;
      const run = j - i;
      if (run > threshold) {
        result.push({
          type: "collapsed-context",
          lines: lines.slice(i, j),
          count: run,
        });
      } else {
        for (let k = i; k < j; k++) {
          result.push({ type: "line", line: lines[k] });
        }
      }
      i = j;
    } else {
      result.push({ type: "line", line: lines[i] });
      i++;
    }
  }

  return result;
}
