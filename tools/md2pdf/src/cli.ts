import { readFile, writeFile } from "node:fs/promises";
import { parseArgs } from "node:util";
import { closeBrowser, htmlToPdf } from "./pdf.js";
import { renderMarkdown } from "./render.js";

function defaultOutputPath(input: string): string {
  const stripped = input.replace(/\.(md|markdown)$/i, "");
  return `${stripped}.pdf`;
}

const { values, positionals } = parseArgs({
  options: {
    output: { type: "string", short: "o" },
  },
  allowPositionals: true,
});

const [input] = positionals;
if (!input) {
  console.error("usage: md2pdf <input.md> [-o <output.pdf>]");
  process.exit(1);
}

const markdown = await readFile(input, "utf8");
const { document } = await renderMarkdown(markdown);
const pdf = await htmlToPdf(document);
await closeBrowser();

const output = values.output ?? defaultOutputPath(input);
await writeFile(output, pdf);
console.log(output);
