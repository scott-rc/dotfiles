# Slides Workspace

pnpm monorepo for Slidev presentations.

## Workspace Structure

- `pnpm-workspace.yaml` defines workspace packages with `talks/*`
- Each talk lives in `talks/<talk-name>/` with its own `package.json` and `slides.md`
- Shared Slidev dependencies use the `catalog:` protocol in `pnpm-workspace.yaml`

## Common Commands

Run all commands from the individual talk directory (`talks/<name>/`):

- `pnpm dev` — start dev server with hot reload
- `pnpm build` — build static SPA output
- `pnpm export` — export slides to PDF (requires playwright)

## Dependencies

- Shared Slidev packages are declared in `pnpm-workspace.yaml` under `catalog:`
- Individual talks reference them with `catalog:` in their own `package.json`
- Run `pnpm install` from the workspace root after adding new dependencies
