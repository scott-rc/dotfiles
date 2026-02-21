# Author

Write or edit Slidev presentation content following Slidev markdown conventions.

## Instructions

1. **Load syntax reference**:
   MUST read [slidev-syntax.md](slidev-syntax.md) before writing any slides. This ensures correct use of frontmatter, layouts, code blocks, click animations, and other Slidev features.

2. **Locate and read**:
   Identify which talk to edit. If the workspace has multiple talks, ask the user which one (list `talks/` directories if needed). Read the current `slides.md` to understand existing content and structure.

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
