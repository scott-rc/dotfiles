import { renderToStaticMarkup } from "react-dom/server";
import Markdown from "react-markdown";
import remarkGfm from "remark-gfm";

export interface RenderedDocument {
  /** Complete standalone HTML document. */
  document: string;
  /** Rendered body markup alone, without the document shell. */
  body: string;
}

export async function renderMarkdown(markdown: string): Promise<RenderedDocument> {
  const body = renderToStaticMarkup(
    <Markdown remarkPlugins={[remarkGfm]}>{markdown}</Markdown>,
  );
  const document = [
    "<!doctype html>",
    '<html lang="en">',
    '<head><meta charset="utf-8"></head>',
    `<body>${body}</body>`,
    "</html>",
  ].join("\n");
  return { document, body };
}
