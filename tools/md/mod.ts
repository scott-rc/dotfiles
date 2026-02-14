import { Lexer } from "marked";
import { test as hasFrontmatter } from "@std/front-matter/test";
import { extract } from "@std/front-matter/yaml";
import { renderFrontmatter, renderTokens, type RenderOptions } from "./render.ts";

/** Render a markdown string into styled terminal output. */
export async function renderMarkdown(
  markdown: string,
  options: RenderOptions,
): Promise<string> {
  let body = markdown;
  let frontmatterBlock = "";

  if (hasFrontmatter(markdown, ["yaml"])) {
    try {
      const { attrs, body: extractedBody } = extract<Record<string, unknown>>(markdown);
      body = extractedBody;
      frontmatterBlock = renderFrontmatter(attrs);
    } catch {
      // Malformed YAML — render the full string as-is
    }
  }

  const tokens = new Lexer().lex(body);
  const rendered = await renderTokens(tokens, options);

  if (frontmatterBlock) {
    return frontmatterBlock + "\n\n" + rendered;
  }
  return rendered;
}

export type { RenderOptions };
