# zellij-sync-stacks

Zellij WASM plugin that synchronizes two side-by-side pane stacks. When navigating up/down in one stack, the other stack expands the pane at the same index — keeping both stacks visually aligned. Also provides synced new-pane creation and synced pane movement in move mode.

Built by the parent `apply.sh` and symlinked to `~/.config/zellij/plugins/zellij-sync-stacks.wasm`, so rebuilding with `cargo build --release` automatically updates the plugin.

## Usage

The plugin is loaded at Zellij startup via `load_plugins` in `config.kdl` and triggered by keybindings:

| Shortcut | Action |
|----------|--------|
| `Cmd+Shift+J` / `Cmd+Shift+Down` | Navigate down in synced stacks |
| `Cmd+Shift+K` / `Cmd+Shift+Up` | Navigate up in synced stacks |
| `Cmd+Shift+N` | New pane (opens 2 in stacked layout, 1 otherwise) |
| Move mode `h/j/k/l` / arrows | Move pane (synced up/down in stacked layout, normal otherwise) |

### Pipe Messages

| Name | Payload | Description |
|------|---------|-------------|
| `navigate` | `"up"` / `"down"` | Synced stack navigation |
| `new_pane` | (none) | Open 1 or 2 terminals depending on layout |
| `move_pane` | `"up"` / `"down"` / `"left"` / `"right"` | Synced pane move (up/down), normal move (left/right) |
| `dump` | (none) | Debug: write plugin state to focused pane stdin |

### Fallback

When the layout isn't two side-by-side stacks (single column, grid, three+ columns, etc.):
- **navigate** falls back to `MoveFocus` up/down
- **new_pane** opens a single terminal (equivalent to `NewPane`)
- **move_pane** falls back to `MovePane` in the given direction

### Debug

Send a `dump` pipe message to write the plugin's internal state (manifest, tab info, detected stacks) to the focused pane's stdin.

## How It Works

1. Subscribes to `PaneUpdate` and `TabUpdate` events to track pane layout
2. Filters panes to selectable, non-plugin, non-floating terminal panes
3. Groups panes by `pane_x` into columns — requires exactly 2 columns, each with 2+ panes and at least one collapsed (rows <= 2)
4. **navigate**: computes the target index in both stacks (clamping for unequal lengths), focuses other stack then current stack
5. **new_pane**: opens 2 terminals if stacks detected, 1 otherwise — the swap layout engine redistributes panes evenly
6. **move_pane** (up/down): finds the counterpart pane at the same index in the other stack, moves both panes in the same direction, then restores focus

## Architecture

Single-file plugin (`main.rs`) split into pure logic and WASM glue:

| Section | Purpose |
|---------|---------|
| Core types | `PaneEntry`, `DetectedStacks`, `NavigationResult`, `SyncedMoveResult` — platform-independent structs |
| Pure functions | `detect_stacks()` — groups panes into two columns, validates stacked layout |
| Pure functions | `compute_navigation()` — resolves target pane IDs for synced navigation |
| Pure functions | `pane_count_for_new()` — returns 2 if stacks detected, 1 otherwise |
| Pure functions | `compute_synced_move()` — resolves pane IDs for synced move (up/down only) |
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

Since `.cargo/config.toml` defaults to `wasm32-wasip1`, you must override the target to run tests natively. Tests exercise all pure functions with various layouts (two stacks, single column, grids, unequal stacks, boundary conditions, filtered plugins/floating panes).
