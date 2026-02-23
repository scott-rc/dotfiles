# Present

Run the dev server, build static output, or export to PDF/PNG.

## Instructions

1. **Identify talk**:
   If ambiguous, discover available talks via `ls ~/Code/personal/slides/talks/` and present them as AskUserQuestion options.

2. **Dev server**:
   Start the live-reload development server for previewing slides:
   ```
   cd ~/Code/personal/slides/talks/<name> && pnpm dev
   ```
   The server runs at `http://localhost:3030` by default. Presenter view is at `http://localhost:3030/presenter`.

3. **Build static SPA**:
   Build an optimized static single-page application:
   ```
   cd ~/Code/personal/slides/talks/<name> && pnpm build
   ```
   Output goes to `dist/`. The result can be deployed to any static hosting.

4. **Export to PDF**:
   Export slides to a PDF file:
   ```
   cd ~/Code/personal/slides/talks/<name> && pnpm export
   ```
   Requires playwright (installed automatically on first run). Output is `slides-export.pdf` in the talk directory.

5. **Export to PNG**:
   Export each slide as a separate PNG image:
   ```
   cd ~/Code/personal/slides/talks/<name> && pnpm export -- --format png
   ```
   Output goes to a `slides-export/` directory with one PNG per slide.
