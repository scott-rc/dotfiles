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
  md <file>       Render a markdown file
  md -            Read from stdin
  md --help       Show this help

Options:
  -w, --width <n>   Set output width (default: min(terminal, ${MAX_WIDTH}))
  --no-color        Disable color output
  --no-pager        Disable built-in pager`);
  Deno.exit(0);
}

if (args["no-color"]) {
  setColorEnabled(false);
}

const file = args._[0] as string | undefined;

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

const renderCentered = async (): Promise<string> => {
  let termWidth: number | null = null;
  try {
    termWidth = Deno.consoleSize().columns;
  } catch {
    // Not a TTY
  }

  const w = args.width
    ? parseInt(args.width, 10)
    : termWidth !== null
      ? Math.min(termWidth, MAX_WIDTH)
      : MAX_WIDTH;

  const rendered = await renderMarkdown(input, { width: w });

  const margin = termWidth !== null
    ? " ".repeat(Math.floor(Math.max(0, termWidth - w) / 2))
    : "";

  return margin
    ? rendered.split("\n").map((line) => margin + line).join("\n")
    : rendered;
};

const centered = await renderCentered();

const shouldPage =
  !args["no-pager"] &&
  Deno.stdout.isTerminal() &&
  file !== undefined &&
  file !== "-";

if (shouldPage) {
  const height = Deno.consoleSize().rows;
  if (centered.split("\n").length > height) {
    const { runPager } = await import("./pager.ts");
    await runPager(centered, { filePath, rawContent: input, onResize: renderCentered });
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
