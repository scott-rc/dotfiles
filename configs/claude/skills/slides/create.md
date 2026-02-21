# Create

Scaffold a new talk package inside the slides workspace.

## Instructions

1. **Interview**:
   Ask the user for:
   - Talk name (MUST be kebab-case, e.g., `intro-to-graphql`)
   - Theme (optional, default: `@slidev/theme-default`)

   Verify `~/Code/personal/slides/` exists. If it does not, run the Setup operation first.

2. **Scaffold talk**:
   Create `talks/<name>/` with two files:
   - `package.json`:
     ```json
     {
       "name": "<name>",
       "private": true,
       "scripts": {
         "dev": "slidev",
         "build": "slidev build",
         "export": "slidev export"
       },
       "dependencies": {
         "@slidev/cli": "catalog:",
         "<theme-package>": "catalog:"
       }
     }
     ```
     If the theme is not `@slidev/theme-default`, MUST also add it to the root `pnpm-workspace.yaml` catalog before referencing it.
   - `slides.md` with starter content:
     ```markdown
     ---
     theme: <theme>
     title: <Title>
     transition: slide-left
     ---

     # <Title>

     Subtitle or author info.

     ---

     # Slide 2

     Content goes here.

     ---
     layout: end
     ---

     # Thank You
     ```

3. **Install and report**:
   - Run `pnpm install` from the workspace root (`~/Code/personal/slides/`)
   - Report the new talk location and how to start the dev server:
     ```
     cd ~/Code/personal/slides/talks/<name> && pnpm dev
     ```
