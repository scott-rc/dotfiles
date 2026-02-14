import { type BundledLanguage, bundledLanguages, codeToTokens } from "shiki";
import { bold, italic, rgb24, underline } from "@std/fmt/colors";

/** Highlight code with syntax colors for a given language.
 *  Returns plain text if no language is given or the language is unrecognized. */
export async function highlightCode(
  code: string,
  lang?: string,
): Promise<string> {
  if (!lang) return code;
  if (!(lang in bundledLanguages)) return code;

  try {
    const { tokens } = await codeToTokens(code, {
      lang: lang as BundledLanguage,
      theme: "github-dark",
    });

    return tokens
      .map((line) =>
        line
          .map((token) => {
            let text = token.content;
            if (token.color) {
              const hex = parseInt(token.color.slice(1), 16);
              text = rgb24(text, hex);
            }
            if (token.fontStyle) {
              if (token.fontStyle & 1) text = italic(text);
              if (token.fontStyle & 2) text = bold(text);
              if (token.fontStyle & 4) text = underline(text);
            }
            return text;
          })
          .join("")
      )
      .join("\n");
  } catch {
    return code;
  }
}
