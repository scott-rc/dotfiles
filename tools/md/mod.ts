import { Lexer } from "marked";
import { renderTokens, type RenderOptions } from "./render.ts";

/** Render a markdown string into styled terminal output. */
export async function renderMarkdown(
  markdown: string,
  options: RenderOptions,
): Promise<string> {
  const tokens = new Lexer().lex(markdown);
  return await renderTokens(tokens, options);
}

export type { RenderOptions };
