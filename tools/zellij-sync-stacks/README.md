# zellij-sync-stacks

Zellij WASM plugin that synchronizes two side-by-side pane stacks. When navigating up/down in one stack, the other stack expands the pane at the same index — keeping both stacks visually aligned.

Built by the parent `apply.sh`, which compiles the WASM binary and copies it to `~/.config/zellij/plugins/zellij-sync-stacks.wasm`.

## Usage

The plugin is loaded at Zellij startup via `load_plugins` in `config.kdl` and triggered by keybindings:

| Shortcut | Action |
|----------|--------|
| `Cmd+Shift+J` / `Cmd+Shift+Down` | Navigate down in synced stacks |
| `Cmd+Shift+K` / `Cmd+Shift+Up` | Navigate up in synced stacks |

These send a `navigate` pipe message with payload `"up"` or `"down"` to the plugin via `MessagePlugin`.

### Fallback

When the layout isn't two side-by-side stacks (single column, grid, three+ columns, etc.), the plugin falls back to Zellij's built-in `MoveFocus` up/down.

### Debug

Send a `dump` pipe message to write the plugin's internal state (manifest, tab info, detected stacks) to the focused pane's stdin.

## How It Works

1. Subscribes to `PaneUpdate` and `TabUpdate` events to track pane layout
2. On `navigate`, filters panes to selectable, non-plugin, non-floating terminal panes
3. Groups panes by `pane_x` into columns — requires exactly 2 columns, each with 2+ panes and at least one collapsed (rows <= 2)
4. Computes the target index in both stacks, clamping if stacks have unequal lengths
5. Focuses the target pane in the *other* stack first, then the target in the *current* stack (restoring focus to the navigated side)

## Architecture

Single-file plugin (`main.rs`) split into pure logic and WASM glue:

| Section | Purpose |
|---------|---------|
| Core types | `PaneEntry`, `DetectedStacks`, `NavigationResult` — platform-independent structs |
| Pure functions | `detect_stacks()` — groups panes into two columns, validates stacked layout |
| Pure functions | `compute_navigation()` — resolves target pane IDs for a given direction |
| WASM glue | `SyncStacksPlugin` — `ZellijPlugin` impl, event handling, pipe dispatch, `dump_state()` |

The pure functions are `#[cfg(test)]`-testable without WASM. The WASM-specific code is gated behind `#[cfg(target_arch = "wasm32")]`.

## Building

```bash
cd tools/zellij-sync-stacks
cargo build --release
```

The `.cargo/config.toml` sets the default target to `wasm32-wasip1`. The output lands at `target/wasm32-wasip1/release/zellij-sync-stacks.wasm`.

## Testing

```bash
cargo test --target aarch64-apple-darwin
```

Since `.cargo/config.toml` defaults to `wasm32-wasip1`, you must override the target to run tests natively. Tests exercise `detect_stacks()` and `compute_navigation()` with various layouts (two stacks, single column, grids, unequal stacks, boundary conditions, filtered plugins/floating panes).
