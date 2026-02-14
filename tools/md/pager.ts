// deno-lint-ignore-file no-control-regex
import { splitAnsi, stripAnsi, visibleLength } from "./wrap.ts";

export interface PagerOptions {
  filePath?: string;
  rawContent?: string;
  onResize?: () => Promise<string>;
}

export interface PagerState {
  lines: string[];
  topLine: number;
  searchQuery: string;
  searchMatches: number[];
  currentMatch: number;
  mode: "normal" | "search";
  searchInput: string;
  searchCursor: number;
  searchMessage: string;
  filePath?: string;
  rawContent?: string;
}

const ESC = "\x1b";
const CSI = `${ESC}[`;
const ALT_SCREEN_ON = `${CSI}?1049h`;
const ALT_SCREEN_OFF = `${CSI}?1049l`;
const CURSOR_HIDE = `${CSI}?25l`;
const CURSOR_SHOW = `${CSI}?25h`;
const CLEAR_LINE = `${CSI}2K`;
const CLEAR_SCREEN = `${CSI}2J`;
const REVERSE = `${CSI}7m`;
const NO_REVERSE = `${CSI}27m`;
const RESET = `${CSI}0m`;
const DIM = `${CSI}2m`;
const NO_DIM = `${CSI}22m`;

const encoder = new TextEncoder();

function write(s: string): void {
  Deno.stdout.writeSync(encoder.encode(s));
}

function moveTo(row: number, col: number): void {
  write(`${CSI}${row + 1};${col + 1}H`);
}

function getTermSize(): { rows: number; cols: number } {
  try {
    const size = Deno.consoleSize();
    return { rows: size.rows, cols: size.columns };
  } catch {
    return { rows: 24, cols: 80 };
  }
}

export function truncateLine(line: string, maxWidth: number): string {
  const segments = splitAnsi(line);
  let visWidth = 0;
  let result = "";

  for (const seg of segments) {
    if (seg.match(/\x1b\[[0-9;]*m/)) {
      result += seg;
      continue;
    }

    if (visWidth + seg.length <= maxWidth) {
      result += seg;
      visWidth += seg.length;
    } else {
      const remaining = maxWidth - visWidth;
      if (remaining > 1) {
        result += seg.slice(0, remaining - 1) + "…";
      } else if (remaining === 1) {
        result += "…";
      }
      visWidth = maxWidth;
      break;
    }
  }

  return result;
}

export function highlightSearch(line: string, query: string): string {
  if (!query) return line;

  const plain = stripAnsi(line);
  const lowerPlain = plain.toLowerCase();
  const lowerQuery = query.toLowerCase();

  // Find all match positions in the plain text
  const matches: { start: number; end: number }[] = [];
  let searchFrom = 0;
  while (searchFrom < lowerPlain.length) {
    const idx = lowerPlain.indexOf(lowerQuery, searchFrom);
    if (idx === -1) break;
    matches.push({ start: idx, end: idx + lowerQuery.length });
    searchFrom = idx + 1;
  }

  if (matches.length === 0) return line;

  // Build a map from visible char index to original string index
  const segments = splitAnsi(line);
  const posMap: number[] = []; // posMap[visibleIdx] = originalIdx
  let origIdx = 0;
  let visIdx = 0;

  for (const seg of segments) {
    if (seg.match(/\x1b\[[0-9;]*m/)) {
      origIdx += seg.length;
      continue;
    }
    for (let i = 0; i < seg.length; i++) {
      posMap[visIdx] = origIdx;
      visIdx++;
      origIdx++;
    }
  }
  // Sentinel for end-of-string insertions
  posMap[visIdx] = origIdx;

  // Inject highlight codes in reverse order to preserve indices
  // Build result by inserting highlight markers
  // Work with string positions, inserting in reverse
  type Insertion = { origPos: number; code: string };
  const insertions: Insertion[] = [];

  for (const m of matches) {
    insertions.push({ origPos: posMap[m.start], code: REVERSE });
    insertions.push({ origPos: posMap[m.end], code: NO_REVERSE });
  }

  // Sort insertions by position descending, so we insert from end to start
  insertions.sort((a, b) => b.origPos - a.origPos);

  let result = line;
  for (const ins of insertions) {
    result = result.slice(0, ins.origPos) + ins.code + result.slice(ins.origPos);
  }

  return result;
}

/** Find the word boundary to the left of the cursor. */
export function wordBoundaryLeft(text: string, cursor: number): number {
  let pos = cursor;
  // Skip spaces
  while (pos > 0 && text[pos - 1] === " ") pos--;
  // Skip non-spaces (the word)
  while (pos > 0 && text[pos - 1] !== " ") pos--;
  return pos;
}

/** Find the word boundary to the right of the cursor. */
export function wordBoundaryRight(text: string, cursor: number): number {
  let pos = cursor;
  // Skip non-spaces (current word)
  while (pos < text.length && text[pos] !== " ") pos++;
  // Skip spaces
  while (pos < text.length && text[pos] === " ") pos++;
  return pos;
}

/** Map a scroll position proportionally when line count changes. */
export function mapScrollPosition(
  oldTopLine: number,
  oldLineCount: number,
  newLineCount: number,
): number {
  if (oldLineCount <= 1 || newLineCount <= 1) return 0;
  const ratio = oldTopLine / (oldLineCount - 1);
  return Math.round(ratio * (newLineCount - 1));
}

/** Find the search match index closest to a given line position. */
export function findNearestMatch(matches: number[], topLine: number): number {
  if (matches.length === 0) return -1;
  const idx = matches.findIndex((m) => m >= topLine);
  return idx === -1 ? matches.length - 1 : idx;
}

export function findMatches(lines: string[], query: string): number[] {
  if (!query) return [];
  const lowerQuery = query.toLowerCase();
  const matches: number[] = [];
  for (let i = 0; i < lines.length; i++) {
    if (stripAnsi(lines[i]).toLowerCase().includes(lowerQuery)) {
      matches.push(i);
    }
  }
  return matches;
}

export interface StatusBarInput {
  mode: "normal" | "search";
  searchInput: string;
  searchCursor: number;
  searchMessage: string;
  searchQuery: string;
  searchMatches: number[];
  currentMatch: number;
  topLine: number;
  lineCount: number;
  contentHeight: number;
  filePath?: string;
}

export function formatStatusBar(input: StatusBarInput, cols: number): string {
  const endLine = Math.min(input.topLine + input.contentHeight, input.lineCount);
  const range = `${input.topLine + 1}-${endLine}/${input.lineCount}`;

  // Position indicator: TOP, END, or percentage
  const atTop = input.topLine === 0;
  const atEnd = endLine >= input.lineCount;
  let position: string;
  if (atTop) {
    position = "TOP";
  } else if (atEnd) {
    position = "END";
  } else {
    const pct = input.lineCount > 0
      ? Math.round((endLine / input.lineCount) * 100)
      : 100;
    position = `${pct}%`;
  }

  // Search input mode: show prompt with cursor block at searchCursor position
  if (input.mode === "search") {
    const before = input.searchInput.slice(0, input.searchCursor);
    const cursorChar = input.searchInput[input.searchCursor] ?? "\u2588";
    const after = input.searchInput.slice(input.searchCursor + 1);
    const prompt = `/${before}${NO_REVERSE}${cursorChar}${REVERSE}${after}`;
    const visLen = 1 + input.searchInput.length + (input.searchCursor >= input.searchInput.length ? 1 : 0);
    const pad = Math.max(0, cols - visLen);
    return prompt + " ".repeat(pad);
  }

  // Search message (e.g. "Copied: file.md"): full-width, no right side
  if (input.searchMessage) {
    return input.searchMessage.padEnd(cols);
  }

  // Build left and right sides
  let left: string;
  if (input.searchQuery && input.searchMatches.length > 0) {
    left = `/${input.searchQuery} (${input.currentMatch + 1}/${input.searchMatches.length})`;
  } else {
    const filename = input.filePath?.split("/").pop() ?? "";
    left = filename;
  }

  const right = `${DIM}${range}${NO_DIM} ${position}`;
  const rightVisible = visibleLength(right);

  // Ensure the bar fits exactly in cols
  const gap = cols - left.length - rightVisible;
  if (gap >= 1) {
    return left + " ".repeat(gap) + right;
  }

  // Terminal too narrow: truncate left to preserve right
  const availLeft = cols - rightVisible - 1;
  if (availLeft >= 1) {
    return left.slice(0, availLeft) + " " + right;
  }

  // Very narrow: just show right, truncated if needed
  return right.padStart(cols);
}

/** Wraps formatStatusBar with ANSI codes: RESET to clear bleed, REVERSE, then RESET at end. */
export function renderStatusBar(input: StatusBarInput, cols: number): string {
  return `${RESET}${REVERSE}${formatStatusBar(input, cols)}${RESET}`;
}

function render(state: PagerState): void {
  const { rows, cols } = getTermSize();
  const contentHeight = rows - 1;

  // Clamp topLine
  const maxTop = Math.max(0, state.lines.length - Math.ceil(contentHeight / 2));
  if (state.topLine > maxTop) state.topLine = maxTop;
  if (state.topLine < 0) state.topLine = 0;

  moveTo(0, 0);

  // Render content lines
  for (let i = 0; i < contentHeight; i++) {
    const lineIdx = state.topLine + i;
    write(CLEAR_LINE);
    if (lineIdx < state.lines.length) {
      let line = state.lines[lineIdx];
      if (state.searchQuery) {
        line = highlightSearch(line, state.searchQuery);
      }
      line = truncateLine(line, cols);
      write(line);
    }
    if (i < contentHeight - 1) write("\r\n");
  }

  // Status bar
  write("\r\n" + CLEAR_LINE);
  write(renderStatusBar({
    mode: state.mode,
    searchInput: state.searchInput,
    searchCursor: state.searchCursor,
    searchMessage: state.searchMessage,
    searchQuery: state.searchQuery,
    searchMatches: state.searchMatches,
    currentMatch: state.currentMatch,
    topLine: state.topLine,
    lineCount: state.lines.length,
    contentHeight,
    filePath: state.filePath,
  }, cols));
}

export type Key =
  | { type: "char"; char: string }
  | { type: "enter" }
  | { type: "escape" }
  | { type: "backspace" }
  | { type: "ctrl-c" }
  | { type: "ctrl-d" }
  | { type: "ctrl-u" }
  | { type: "up" }
  | { type: "down" }
  | { type: "left" }
  | { type: "right" }
  | { type: "alt-left" }
  | { type: "alt-right" }
  | { type: "alt-backspace" }
  | { type: "pageup" }
  | { type: "pagedown" }
  | { type: "home" }
  | { type: "end" }
  | { type: "unknown" };

export function parseKey(buf: Uint8Array): Key {
  if (buf.length === 0) return { type: "unknown" };

  // ESC sequence
  if (buf[0] === 0x1b) {
    if (buf.length === 1) return { type: "escape" };
    if (buf[1] === 0x5b) {
      // CSI sequence
      if (buf.length >= 3) {
        switch (buf[2]) {
          case 0x41: return { type: "up" };    // A
          case 0x42: return { type: "down" };  // B
          case 0x43: return { type: "right" }; // C
          case 0x44: return { type: "left" };  // D
          case 0x48: return { type: "home" };  // H
          case 0x46: return { type: "end" };   // F
          case 0x35: // 5~
            if (buf.length >= 4 && buf[3] === 0x7e) return { type: "pageup" };
            break;
          case 0x36: // 6~
            if (buf.length >= 4 && buf[3] === 0x7e) return { type: "pagedown" };
            break;
          case 0x31: // CSI 1;3C / CSI 1;3D (alt-right / alt-left)
            if (buf.length >= 6 && buf[3] === 0x3b && buf[4] === 0x33) {
              if (buf[5] === 0x43) return { type: "alt-right" };
              if (buf[5] === 0x44) return { type: "alt-left" };
            }
            break;
        }
      }
    }
    // ESC b (alt-left), ESC f (alt-right), ESC DEL (alt-backspace)
    if (buf[1] === 0x62) return { type: "alt-left" };
    if (buf[1] === 0x66) return { type: "alt-right" };
    if (buf[1] === 0x7f) return { type: "alt-backspace" };
    return { type: "unknown" };
  }

  // Single byte
  switch (buf[0]) {
    case 0x03: return { type: "ctrl-c" };
    case 0x04: return { type: "ctrl-d" };
    case 0x0d: return { type: "enter" };
    case 0x15: return { type: "ctrl-u" };
    case 0x7f: return { type: "backspace" };
    default:
      if (buf[0] >= 0x20 && buf[0] <= 0x7e) {
        return { type: "char", char: String.fromCharCode(buf[0]) };
      }
      return { type: "unknown" };
  }
}

export function handleSearchKey(state: PagerState, key: Key): void {
  switch (key.type) {
    case "char":
      state.searchInput = state.searchInput.slice(0, state.searchCursor) + key.char + state.searchInput.slice(state.searchCursor);
      state.searchCursor++;
      break;
    case "backspace":
      if (state.searchCursor > 0) {
        state.searchInput = state.searchInput.slice(0, state.searchCursor - 1) + state.searchInput.slice(state.searchCursor);
        state.searchCursor--;
      }
      if (!state.searchInput) {
        state.mode = "normal";
      }
      break;
    case "alt-backspace": {
      const boundary = wordBoundaryLeft(state.searchInput, state.searchCursor);
      state.searchInput = state.searchInput.slice(0, boundary) + state.searchInput.slice(state.searchCursor);
      state.searchCursor = boundary;
      if (!state.searchInput) {
        state.mode = "normal";
      }
      break;
    }
    case "ctrl-u": {
      state.searchInput = state.searchInput.slice(state.searchCursor);
      state.searchCursor = 0;
      if (!state.searchInput) {
        state.mode = "normal";
      }
      break;
    }
    case "left":
      if (state.searchCursor > 0) state.searchCursor--;
      break;
    case "right":
      if (state.searchCursor < state.searchInput.length) state.searchCursor++;
      break;
    case "alt-left":
      state.searchCursor = wordBoundaryLeft(state.searchInput, state.searchCursor);
      break;
    case "alt-right":
      state.searchCursor = wordBoundaryRight(state.searchInput, state.searchCursor);
      break;
    case "enter": {
      state.mode = "normal";
      if (state.searchInput) {
        state.searchQuery = state.searchInput;
        state.searchMatches = findMatches(state.lines, state.searchQuery);
        if (state.searchMatches.length > 0) {
          let found = state.searchMatches.findIndex((m) => m >= state.topLine);
          if (found === -1) found = 0;
          state.currentMatch = found;
          state.searchMessage = "";
          scrollToMatch(state);
        } else {
          state.currentMatch = -1;
          state.searchMessage = `Pattern not found: ${state.searchQuery}`;
        }
      }
      state.searchInput = "";
      state.searchCursor = 0;
      break;
    }
    case "escape":
    case "ctrl-c":
      state.mode = "normal";
      state.searchInput = "";
      state.searchCursor = 0;
      break;
  }
}

function scrollToMatch(state: PagerState): void {
  if (state.searchMatches.length === 0) return;
  const matchLine = state.searchMatches[state.currentMatch];
  const { rows } = getTermSize();
  const contentHeight = rows - 1;

  // Center the match on screen if possible
  const target = matchLine - Math.floor(contentHeight / 3);
  state.topLine = Math.max(0, Math.min(target, state.lines.length - Math.ceil(contentHeight / 2)));
}

export function mapToSourceLine(
  topLine: number,
  renderedLineCount: number,
  rawContent: string,
): number {
  const sourceLineCount = rawContent.split("\n").length;
  return Math.round((topLine / renderedLineCount) * sourceLineCount) + 1;
}

async function copyToClipboard(text: string): Promise<boolean> {
  try {
    const cmd = new Deno.Command("pbcopy", {
      stdin: "piped",
      stdout: "null",
      stderr: "null",
    });
    const proc = cmd.spawn();
    const writer = proc.stdin.getWriter();
    await writer.write(new TextEncoder().encode(text));
    await writer.close();
    const { success } = await proc.status;
    return success;
  } catch {
    return false;
  }
}

async function openInEditor(filePath: string, line?: number): Promise<void> {
  const editor = Deno.env.get("EDITOR") || "nvim";
  const basename = editor.split("/").pop() ?? editor;
  const isVim = ["vim", "nvim"].includes(basename);
  const args: string[] = [];
  if (isVim) args.push("-R");
  if (isVim && line) args.push(`+${line}`);
  args.push(filePath);
  const cmd = new Deno.Command(editor, {
    args,
    stdin: "inherit",
    stdout: "inherit",
    stderr: "inherit",
  });
  const proc = cmd.spawn();
  await proc.status;
}

export async function runPager(
  content: string,
  options?: PagerOptions,
): Promise<void> {
  const state: PagerState = {
    lines: content.split("\n"),
    topLine: 0,
    searchQuery: "",
    searchMatches: [],
    currentMatch: -1,
    mode: "normal",
    searchInput: "",
    searchCursor: 0,
    searchMessage: "",
    filePath: options?.filePath,
    rawContent: options?.rawContent,
  };

  // Enter alternate screen and hide cursor
  write(ALT_SCREEN_ON + CURSOR_HIDE);
  Deno.stdin.setRaw(true);

  let resizing = false;
  let pendingResize = false;
  let paused = false;

  const doResize = () => {
    if (!options?.onResize) {
      write(CLEAR_SCREEN);
      render(state);
      return;
    }
    resizing = true;
    pendingResize = false;
    const oldLineCount = state.lines.length;
    const oldTopLine = state.topLine;
    options.onResize().then((newContent) => {
      state.lines = newContent.split("\n");
      state.topLine = mapScrollPosition(oldTopLine, oldLineCount, state.lines.length);
      if (state.searchQuery) {
        state.searchMatches = findMatches(state.lines, state.searchQuery);
        state.currentMatch = findNearestMatch(state.searchMatches, state.topLine);
      }
      write(CLEAR_SCREEN);
      render(state);
    }).catch(() => {
      write(CLEAR_SCREEN);
      render(state);
    }).finally(() => {
      resizing = false;
      if (pendingResize) {
        pendingResize = false;
        doResize();
      }
    });
  };

  const handleResize = () => {
    if (paused) return;
    if (resizing) {
      pendingResize = true;
      return;
    }
    doResize();
  };

  Deno.addSignalListener("SIGWINCH", handleResize);

  try {
    render(state);

    let reader = Deno.stdin.readable.getReader();
    try {
      while (true) {
        const { value, done } = await reader.read();
        if (done || !value) break;

        const key = parseKey(value);
        const { rows } = getTermSize();
        const contentHeight = rows - 1;
        const halfPage = Math.max(1, Math.floor(contentHeight / 2));

        if (state.mode === "search") {
          handleSearchKey(state, key);
        } else {
          // Normal mode
          state.searchMessage = "";

          switch (key.type) {
            case "char":
              switch (key.char) {
                case "q":
                  return;
                case "j":
                  state.topLine = Math.min(state.topLine + 1, Math.max(0, state.lines.length - Math.ceil(contentHeight / 2)));
                  break;
                case "k":
                  state.topLine = Math.max(state.topLine - 1, 0);
                  break;
                case "d":
                case " ":
                  state.topLine = Math.min(state.topLine + halfPage, Math.max(0, state.lines.length - Math.ceil(contentHeight / 2)));
                  break;
                case "u":
                  state.topLine = Math.max(state.topLine - halfPage, 0);
                  break;
                case "g":
                  state.topLine = 0;
                  break;
                case "G":
                  state.topLine = Math.max(0, state.lines.length - Math.ceil(contentHeight / 2));
                  break;
                case "/":
                  state.mode = "search";
                  state.searchInput = "";
                  state.searchCursor = 0;
                  break;
                case "n":
                  if (state.searchMatches.length > 0) {
                    state.currentMatch = (state.currentMatch + 1) % state.searchMatches.length;
                    scrollToMatch(state);
                  }
                  break;
                case "N":
                  if (state.searchMatches.length > 0) {
                    state.currentMatch = (state.currentMatch - 1 + state.searchMatches.length) % state.searchMatches.length;
                    scrollToMatch(state);
                  }
                  break;
                case "c": {
                  if (state.filePath) {
                    const cwd = Deno.cwd();
                    const rel = state.filePath.startsWith(cwd + "/")
                      ? state.filePath.slice(cwd.length + 1)
                      : state.filePath;
                    if (await copyToClipboard(rel)) {
                      state.searchMessage = `Copied: ${rel}`;
                    }
                  } else {
                    state.searchMessage = "No file path available";
                  }
                  break;
                }
                case "C":
                  if (state.filePath) {
                    if (await copyToClipboard(state.filePath)) {
                      state.searchMessage = `Copied: ${state.filePath}`;
                    }
                  } else {
                    state.searchMessage = "No file path available";
                  }
                  break;
                case "y":
                  if (state.rawContent) {
                    if (await copyToClipboard(state.rawContent)) {
                      state.searchMessage = "Copied document to clipboard";
                    }
                  } else {
                    state.searchMessage = "No content available";
                  }
                  break;
                case "v":
                  if (state.filePath) {
                    const sourceLine = state.rawContent && state.lines.length > 0
                      ? mapToSourceLine(state.topLine, state.lines.length, state.rawContent)
                      : undefined;
                    paused = true;
                    reader.releaseLock();
                    Deno.stdin.setRaw(false);
                    write(CURSOR_SHOW + ALT_SCREEN_OFF);
                    await openInEditor(state.filePath, sourceLine);
                    write(ALT_SCREEN_ON + CURSOR_HIDE);
                    Deno.stdin.setRaw(true);
                    reader = Deno.stdin.readable.getReader();
                    paused = false;
                  } else {
                    state.searchMessage = "No file path available";
                  }
                  break;
              }
              break;
            case "ctrl-c":
              return;
            case "ctrl-d":
              state.topLine = Math.min(state.topLine + halfPage, Math.max(0, state.lines.length - Math.ceil(contentHeight / 2)));
              break;
            case "ctrl-u":
              state.topLine = Math.max(state.topLine - halfPage, 0);
              break;
            case "enter":
            case "down":
              state.topLine = Math.min(state.topLine + 1, Math.max(0, state.lines.length - Math.ceil(contentHeight / 2)));
              break;
            case "up":
              state.topLine = Math.max(state.topLine - 1, 0);
              break;
            case "pagedown":
              state.topLine = Math.min(state.topLine + halfPage, Math.max(0, state.lines.length - Math.ceil(contentHeight / 2)));
              break;
            case "pageup":
              state.topLine = Math.max(state.topLine - halfPage, 0);
              break;
            case "home":
              state.topLine = 0;
              break;
            case "end":
              state.topLine = Math.max(0, state.lines.length - Math.ceil(contentHeight / 2));
              break;
          }
        }

        render(state);
      }
    } finally {
      reader.releaseLock();
    }
  } finally {
    Deno.removeSignalListener("SIGWINCH", handleResize);
    Deno.stdin.setRaw(false);
    write(CURSOR_SHOW + ALT_SCREEN_OFF);
  }
}
