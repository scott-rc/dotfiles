import { parseArgs } from "@std/cli/parse-args";
import { setColorEnabled } from "@std/fmt/colors";
import { renderMarkdown } from "./mod.ts";

const MAX_WIDTH = 80;

const args = parseArgs(Deno.args, {
  boolean: ["help", "no-color", "no-pager"],
  string: ["width"],
  alias: { h: "help", w: "width" },
});

if (args.help) {
  console.log(`md — terminal markdown renderer

Usage:
  md <file>         Render a markdown file
  md <directory>    Browse markdown files with fzf
  md -              Read from stdin
  md --help         Show this help

Options:
  -w, --width <n>   Set output width (default: min(terminal, ${MAX_WIDTH}))
  --no-color        Disable color output
  --no-pager        Disable built-in pager

Environment:
  MD_FIND_CMD       Custom find command (default: find {dir} ... *.md *.mdx)
  MD_PICK_CMD       Custom pick command (default: fzf)`);
  Deno.exit(0);
}

if (args["no-color"]) {
  setColorEnabled(false);
}

const file = args._[0] as string | undefined;

function renderWidth(): { width: number; termWidth: number | null } {
  let termWidth: number | null = null;
  try {
    termWidth = Deno.consoleSize().columns;
  } catch {
    // Not a TTY
  }

  const width = args.width
    ? parseInt(args.width, 10)
    : termWidth !== null
      ? Math.min(termWidth, MAX_WIDTH)
      : MAX_WIDTH;

  return { width, termWidth };
}

async function renderCentered(input: string): Promise<string> {
  const { width, termWidth } = renderWidth();
  const rendered = await renderMarkdown(input, { width });

  const margin = termWidth !== null
    ? " ".repeat(Math.floor(Math.max(0, termWidth - width) / 2))
    : "";

  return margin
    ? rendered.split("\n").map((line) => margin + line).join("\n")
    : rendered;
}

async function viewFile(
  path: string,
  opts?: { browsing?: boolean },
): Promise<void> {
  const input = await Deno.readTextFile(path);
  const filePath = await Deno.realPath(path);
  const centered = await renderCentered(input);

  const { shouldPage } = await import("./browse.ts");
  if (
    shouldPage({
      noPager: !!args["no-pager"],
      isTTY: Deno.stdout.isTerminal(),
      contentLines: centered.split("\n").length,
      terminalRows: Deno.stdout.isTerminal() ? Deno.consoleSize().rows : 0,
      browsing: !!opts?.browsing,
    })
  ) {
    const { runPager } = await import("./pager.ts");
    await runPager(centered, {
      filePath,
      rawContent: input,
      onResize: () => renderCentered(input),
    });
    return;
  }

  console.log(centered);
}

// Directory browsing
if (file && file !== "-") {
  try {
    const stat = await Deno.stat(file);
    if (stat.isDirectory) {
      const { browseDirectory } = await import("./browse.ts");
      await browseDirectory(file, (p) => viewFile(p, { browsing: true }), {
        findCmd: Deno.env.get("MD_FIND_CMD"),
        pickCmd: Deno.env.get("MD_PICK_CMD"),
      });
      Deno.exit(0);
    }
  } catch (e) {
    if (e instanceof Deno.errors.NotFound) {
      console.error(`md: ${file}: not found`);
      Deno.exit(1);
    }
    throw e;
  }
}

// File / stdin rendering
let input: string;
let filePath: string | undefined;

if (file === "-" || (!file && !Deno.stdin.isTerminal())) {
  input = new TextDecoder().decode(await readStdin());
} else if (file) {
  input = await Deno.readTextFile(file);
  filePath = await Deno.realPath(file);
} else {
  console.error("Usage: md <file> or md -");
  Deno.exit(1);
}

const centered = await renderCentered(input);

{
  const { shouldPage } = await import("./browse.ts");
  const isTTY = Deno.stdout.isTerminal();
  if (
    file !== undefined &&
    file !== "-" &&
    shouldPage({
      noPager: !!args["no-pager"],
      isTTY,
      contentLines: centered.split("\n").length,
      terminalRows: isTTY ? Deno.consoleSize().rows : 0,
      browsing: false,
    })
  ) {
    const { runPager } = await import("./pager.ts");
    await runPager(centered, {
      filePath,
      rawContent: input,
      onResize: () => renderCentered(input),
    });
    Deno.exit(0);
  }
}

console.log(centered);

async function readStdin(): Promise<Uint8Array> {
  const chunks: Uint8Array[] = [];
  for await (const chunk of Deno.stdin.readable) {
    chunks.push(chunk);
  }
  const total = chunks.reduce((n, c) => n + c.length, 0);
  const result = new Uint8Array(total);
  let offset = 0;
  for (const chunk of chunks) {
    result.set(chunk, offset);
    offset += chunk.length;
  }
  return result;
}
