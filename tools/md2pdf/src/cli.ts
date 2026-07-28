import { readFile, writeFile } from "node:fs/promises";
import { parseArgs } from "node:util";
import { closeBrowser, htmlToPdf } from "./pdf.js";
import { renderMarkdown } from "./render.js";

const usage = "usage: md2pdf <input.md> [-o <output.pdf>]";

function defaultOutputPath(input: string): string {
  const stripped = input.replace(/\.(md|markdown)$/i, "");
  return `${stripped}.pdf`;
}

function fail(message: string): never {
  console.error(message);
  process.exit(1);
}

let values: { output?: string };
let positionals: string[];
try {
  ({ values, positionals } = parseArgs({
    options: {
      output: { type: "string", short: "o" },
    },
    allowPositionals: true,
  }));
} catch {
  fail(usage);
}

if (positionals.length !== 1) fail(usage);
const input = positionals[0];

let markdown: string;
try {
  markdown = await readFile(input, "utf8");
} catch (err) {
  fail(`md2pdf: ${input}: ${err instanceof Error ? err.message : String(err)}`);
}

try {
  const { document } = await renderMarkdown(markdown);
  const pdf = await htmlToPdf(document);
  const output = values.output ?? defaultOutputPath(input);
  await writeFile(output, pdf);
  console.log(output);
} finally {
  await closeBrowser();
}
