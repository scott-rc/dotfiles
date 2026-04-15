import { describe, it, expect } from "vitest";
import { groupContextLines, type GroupedItem } from "../utils/grouping";
import type { WebDiffLine } from "../utils/types";

function makeLine(kind: "context" | "added" | "deleted", idx: number): WebDiffLine {
  return {
    kind,
    content_html: `<span>${kind} ${idx}</span>`,
    raw_content: `${kind} ${idx}`,
    old_lineno: kind === "added" ? null : idx,
    new_lineno: kind === "deleted" ? null : idx,
    line_idx: idx,
  };
}

function makeContextLines(count: number, startIdx = 0): WebDiffLine[] {
  return Array.from({ length: count }, (_, i) => makeLine("context", startIdx + i));
}

describe("groupContextLines", () => {
  it("returns empty array for empty input", () => {
    expect(groupContextLines([])).toEqual([]);
  });

  it("keeps individual lines when context run <= threshold", () => {
    const lines = makeContextLines(3);
    const result = groupContextLines(lines);
    expect(result).toHaveLength(3);
    expect(result.every((r) => r.type === "line")).toBe(true);
  });

  it("keeps exactly threshold lines as individual", () => {
    const lines = makeContextLines(3);
    const result = groupContextLines(lines, 3);
    expect(result).toHaveLength(3);
    expect(result.every((r) => r.type === "line")).toBe(true);
  });

  it("collapses context run > threshold into a single group", () => {
    const lines = makeContextLines(5);
    const result = groupContextLines(lines, 3);
    expect(result).toHaveLength(1);
    expect(result[0].type).toBe("collapsed-context");
    if (result[0].type === "collapsed-context") {
      expect(result[0].count).toBe(5);
      expect(result[0].lines).toHaveLength(5);
    }
  });

  it("passes through non-context lines unchanged", () => {
    const lines = [makeLine("added", 1), makeLine("deleted", 2), makeLine("added", 3)];
    const result = groupContextLines(lines);
    expect(result).toHaveLength(3);
    expect(result.every((r) => r.type === "line")).toBe(true);
  });

  it("handles mixed: context > threshold surrounded by changes", () => {
    const lines = [
      makeLine("added", 1),
      ...makeContextLines(6, 10),
      makeLine("deleted", 20),
    ];
    const result = groupContextLines(lines);
    expect(result).toHaveLength(3); // added + collapsed-context + deleted
    expect(result[0].type).toBe("line");
    expect(result[1].type).toBe("collapsed-context");
    expect(result[2].type).toBe("line");
    if (result[1].type === "collapsed-context") {
      expect(result[1].count).toBe(6);
    }
  });

  it("handles mixed: short context runs stay individual", () => {
    const lines = [
      makeLine("added", 1),
      ...makeContextLines(2, 10),
      makeLine("deleted", 20),
    ];
    const result = groupContextLines(lines);
    // added + 2 context lines + deleted = 4
    expect(result).toHaveLength(4);
    expect(result.every((r) => r.type === "line")).toBe(true);
  });

  it("groups multiple separate context runs independently", () => {
    const lines = [
      ...makeContextLines(5, 1),   // collapsed
      makeLine("added", 10),
      ...makeContextLines(4, 20),  // collapsed
      makeLine("deleted", 30),
      ...makeContextLines(2, 40),  // kept individual
    ];
    const result = groupContextLines(lines);
    // collapsed + added + collapsed + deleted + 2 individual = 6
    expect(result).toHaveLength(6);
    expect(result[0].type).toBe("collapsed-context");
    expect(result[1].type).toBe("line");
    expect(result[2].type).toBe("collapsed-context");
    expect(result[3].type).toBe("line");
    expect(result[4].type).toBe("line");
    expect(result[5].type).toBe("line");
  });

  it("handles all-context input above threshold", () => {
    const lines = makeContextLines(10);
    const result = groupContextLines(lines);
    expect(result).toHaveLength(1);
    expect(result[0].type).toBe("collapsed-context");
    if (result[0].type === "collapsed-context") {
      expect(result[0].count).toBe(10);
    }
  });

  it("handles single context line", () => {
    const lines = makeContextLines(1);
    const result = groupContextLines(lines);
    expect(result).toHaveLength(1);
    expect(result[0].type).toBe("line");
  });

  it("handles single non-context line", () => {
    const lines = [makeLine("added", 1)];
    const result = groupContextLines(lines);
    expect(result).toHaveLength(1);
    expect(result[0].type).toBe("line");
  });

  it("preserves line data in collapsed groups", () => {
    const lines = makeContextLines(4);
    const result = groupContextLines(lines);
    expect(result).toHaveLength(1);
    if (result[0].type === "collapsed-context") {
      expect(result[0].lines[0].raw_content).toBe("context 0");
      expect(result[0].lines[3].raw_content).toBe("context 3");
    }
  });

  it("uses default threshold of 3", () => {
    // 3 context lines: not collapsed (default threshold=3, >3 needed)
    expect(groupContextLines(makeContextLines(3)).every((r) => r.type === "line")).toBe(true);
    // 4 context lines: collapsed
    expect(groupContextLines(makeContextLines(4))[0].type).toBe("collapsed-context");
  });
});
