import { readFile } from "node:fs/promises";
import { createRequire } from "node:module";
import { fileURLToPath } from "node:url";
import { compile } from "@tailwindcss/node";
import { renderToStaticMarkup } from "react-dom/server";
import Markdown from "react-markdown";
import remarkBreaks from "remark-breaks";
import remarkGfm from "remark-gfm";

export interface RenderedDocument {
  /** Complete standalone HTML document. */
  document: string;
  /** Rendered body markup alone, without the document shell. */
  body: string;
}

const require = createRequire(import.meta.url);
const moduleDir = fileURLToPath(new URL(".", import.meta.url));

const proseClasses = "prose prose-sm max-w-none";

const tailwindInput = `
@import "tailwindcss";
@plugin "@tailwindcss/typography";
`;

// The typography plugin's defaults request 500 for links, 800 for h1,
// 600-700 for other headings and strong, and italic for emphasis. An
// undercovered set would make Chromium silently synthesize faux faces.
const interFaces = [
  { weight: 400, style: "normal" },
  { weight: 500, style: "normal" },
  { weight: 600, style: "normal" },
  { weight: 700, style: "normal" },
  { weight: 800, style: "normal" },
  { weight: 400, style: "italic" },
];

// Resume tuning: tighter vertical rhythm than stock prose-sm, plus print
// rules keeping headings attached to the content that follows them.
const tuningCss = `
body {
  font-family: "Inter", sans-serif;
}
.prose h1 { margin-bottom: 0.25em; }
.prose h2 { margin-top: 1.4em; margin-bottom: 0.5em; }
.prose h3 { margin-top: 1.1em; margin-bottom: 0.3em; }
.prose p { margin-top: 0.4em; margin-bottom: 0.4em; }
.prose ul, .prose ol { margin-top: 0.4em; margin-bottom: 0.4em; }
.prose li { margin-top: 0.15em; margin-bottom: 0.15em; }
.prose h1, .prose h2, .prose h3, .prose h4 {
  break-after: avoid;
  break-inside: avoid;
}
`;

async function interFontCss(): Promise<string> {
  const rules = await Promise.all(
    interFaces.map(async ({ weight, style }) => {
      const file = require.resolve(
        `@fontsource/inter/files/inter-latin-${weight}-${style}.woff2`,
      );
      const woff2 = await readFile(file);
      return [
        "@font-face {",
        '  font-family: "Inter";',
        `  font-style: ${style};`,
        `  font-weight: ${weight};`,
        `  src: url(data:font/woff2;base64,${woff2.toString("base64")}) format("woff2");`,
        "}",
      ].join("\n");
    }),
  );
  return rules.join("\n");
}

let stylesheetPromise: Promise<string> | undefined;

function stylesheet(): Promise<string> {
  stylesheetPromise ??= (async () => {
    const compiler = await compile(tailwindInput, {
      base: moduleDir,
      onDependency: () => {},
    });
    const tailwind = compiler.build(proseClasses.split(" "));
    return [tailwind, await interFontCss(), tuningCss].join("\n");
  })();
  return stylesheetPromise;
}

export async function renderMarkdown(markdown: string): Promise<RenderedDocument> {
  const body = renderToStaticMarkup(
    <div className={proseClasses}>
      <Markdown remarkPlugins={[remarkGfm, remarkBreaks]}>{markdown}</Markdown>
    </div>,
  );
  const document = [
    "<!doctype html>",
    '<html lang="en">',
    "<head>",
    '<meta charset="utf-8">',
    `<style>${await stylesheet()}</style>`,
    "</head>",
    `<body>${body}</body>`,
    "</html>",
  ].join("\n");
  return { document, body };
}
