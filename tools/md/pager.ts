// deno-lint-ignore-file no-control-regex
import { splitAnsi, stripAnsi } from "./wrap.ts";

export interface PagerOptions {
  filePath?: string;
  rawContent?: string;
}

interface PagerState {
  lines: string[];
  topLine: number;
  searchQuery: string;
  searchMatches: number[];
  currentMatch: number;
  mode: "normal" | "search";
  searchInput: string;
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
const REVERSE = `${CSI}7m`;
const NO_REVERSE = `${CSI}27m`;
const RESET = `${CSI}0m`;

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
  const endLine = Math.min(state.topLine + contentHeight, state.lines.length);
  const pct = state.lines.length > 0
    ? Math.round((endLine / state.lines.length) * 100)
    : 100;

  let statusText: string;
  if (state.mode === "search") {
    statusText = `/${state.searchInput}`;
  } else if (state.searchMessage) {
    statusText = state.searchMessage;
  } else if (state.searchQuery && state.searchMatches.length > 0) {
    statusText =
      `/${state.searchQuery} (${state.currentMatch + 1}/${state.searchMatches.length})  lines ${state.topLine + 1}-${endLine}/${state.lines.length} ${pct}%`;
  } else {
    statusText = `lines ${state.topLine + 1}-${endLine}/${state.lines.length} ${pct}%`;
  }

  // Pad status to full width, render in reverse video
  const padded = statusText.padEnd(cols).slice(0, cols);
  write(`${REVERSE}${padded}${RESET}`);
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
          case 0x48: return { type: "home" };  // H
          case 0x46: return { type: "end" };   // F
          case 0x35: // 5~
            if (buf.length >= 4 && buf[3] === 0x7e) return { type: "pageup" };
            break;
          case 0x36: // 6~
            if (buf.length >= 4 && buf[3] === 0x7e) return { type: "pagedown" };
            break;
        }
      }
    }
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
    searchMessage: "",
    filePath: options?.filePath,
    rawContent: options?.rawContent,
  };

  // Enter alternate screen and hide cursor
  write(ALT_SCREEN_ON + CURSOR_HIDE);
  Deno.stdin.setRaw(true);

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
          switch (key.type) {
            case "char":
              state.searchInput += key.char;
              break;
            case "backspace":
              state.searchInput = state.searchInput.slice(0, -1);
              break;
            case "enter": {
              state.mode = "normal";
              if (state.searchInput) {
                state.searchQuery = state.searchInput;
                state.searchMatches = findMatches(state.lines, state.searchQuery);
                if (state.searchMatches.length > 0) {
                  // Find first match at or after topLine
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
              break;
            }
            case "escape":
            case "ctrl-c":
              state.mode = "normal";
              state.searchInput = "";
              break;
          }
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
                    reader.releaseLock();
                    Deno.stdin.setRaw(false);
                    write(CURSOR_SHOW + ALT_SCREEN_OFF);
                    await openInEditor(state.filePath, sourceLine);
                    write(ALT_SCREEN_ON + CURSOR_HIDE);
                    Deno.stdin.setRaw(true);
                    reader = Deno.stdin.readable.getReader();
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
    // Restore terminal state
    Deno.stdin.setRaw(false);
    write(CURSOR_SHOW + ALT_SCREEN_OFF);
  }
}
