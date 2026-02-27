# Create

Scaffold a new talk package inside the slides workspace.

## Instructions

1. **Check workspace**:
   Verify `~/Code/personal/slides/` exists. If it does not, run the Setup operation first.

2. **Interview**:
   Ask the user for:
   - Talk name (MUST be kebab-case) -- suggest 1-3 names inferred from context via AskUserQuestion
   - Theme -- present via AskUserQuestion: "@slidev/theme-default (Recommended)", "@slidev/theme-seriph", "@slidev/theme-apple-basic"

3. **Scaffold talk**:
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

4. **Install and verify**:
   - Run `pnpm install` from the workspace root (`~/Code/personal/slides/`)
   - Confirm `talks/<name>/slides.md` and `talks/<name>/package.json` exist, and that `pnpm install` exited successfully.
   - Report the new talk location and how to start the dev server:
     ```
     cd ~/Code/personal/slides/talks/<name> && pnpm dev
     ```
