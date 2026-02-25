# Author

Write or edit Slidev presentation content following Slidev markdown conventions.

## Instructions

1. **Syntax reference**:
   Refer to [slidev-syntax.md](slidev-syntax.md) for layout names, click animation syntax, and advanced features as needed. Do not pre-load the full reference — consult it when you need specific syntax details.

2. **Locate and read**:
   Identify which talk to edit. If the workspace has multiple talks, discover `talks/` directories and present them as AskUserQuestion options. Read the current `slides.md` to understand existing content and structure.

3. **Write/edit slides**:
   Follow Slidev conventions when writing or modifying slide content:
   - Headmatter at the top of the file for global config (theme, title, transition)
   - `---` separators between slides, MUST have blank lines before and after
   - Use built-in layouts (`cover`, `center`, `two-cols`, `section`, `image-right`, `end`, etc.) before creating custom ones
   - Add presenter notes using `<!-- -->` HTML comments at the end of each slide for talking points
   - Use Shiki features in code blocks: language tag, line highlighting (`{1,3-5}`), click-based progressive highlighting (`{1|3-5|7}`)
   - Use `v-click` and `v-clicks` for progressive content reveal
   - Use Mermaid fenced blocks for diagrams
   - SHOULD keep one concept per slide
   - SHOULD use concise text -- slides are visual aids, not documents

4. **Validate**:
   Review the final `slides.md` to ensure:
   - All `---` separators have blank lines around them
   - Headmatter is valid YAML
   - No broken syntax (unclosed fences, malformed frontmatter, mismatched HTML tags)
   - Layout names match Slidev built-in layouts or custom components
   - Presenter notes use correct `<!-- -->` syntax

5. **Report**:
   Report the total slide count and the file path of the updated `slides.md`.
