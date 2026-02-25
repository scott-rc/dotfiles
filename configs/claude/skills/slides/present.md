# Present Slides

Run dev server, build static output, or export to PDF/PNG.

## Instructions

1. **Identify talk**: if ambiguous, list available talks and present as AskUserQuestion options.
2. **Run by intent**:
   - **Dev server**: `cd <talk> && pnpm dev` -- serves at `http://localhost:3030`
   - **Build**: `cd <talk> && pnpm build` -- output to `dist/`
   - **Export PDF**: `cd <talk> && pnpm export` -- requires playwright
   - **Export PNG**: `cd <talk> && pnpm export -- --format png`
3. **Handle playwright errors**: if `pnpm export` exits non-zero, run `pnpm exec playwright --version`. If that also fails, surface install instructions to the user: `pnpm exec playwright install`.
4. **Report**: server URL, output path, or output directory.
