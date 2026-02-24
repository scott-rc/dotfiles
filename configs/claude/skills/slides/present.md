# Present

Run the dev server, build static output, or export to PDF/PNG.

## Instructions

1. **Identify talk**:
   If ambiguous, discover available talks via `ls ~/Code/personal/slides/talks/` and present them as AskUserQuestion options.

2. **Run the requested action**:

   Based on the user's intent, run one of the following:

   - **Dev server** — start live-reload preview:
     ```
     cd ~/Code/personal/slides/talks/<name> && pnpm dev
     ```
     Server runs at `http://localhost:3030`. Presenter view at `http://localhost:3030/presenter`.

   - **Build static SPA** — optimized static output:
     ```
     cd ~/Code/personal/slides/talks/<name> && pnpm build
     ```
     Output goes to `dist/`.

   - **Export to PDF**:
     ```
     cd ~/Code/personal/slides/talks/<name> && pnpm export
     ```
     Requires playwright (run `pnpm exec playwright install` if export fails). Output is `slides-export.pdf`.

   - **Export to PNG** — one image per slide:
     ```
     cd ~/Code/personal/slides/talks/<name> && pnpm export -- --format png
     ```
     Output goes to `slides-export/`.

3. **Report results**: Confirm the action completed — report the server URL, output file path, or output directory as appropriate.
