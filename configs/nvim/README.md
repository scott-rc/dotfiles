# Neovim Config

## Structure

```
configs/nvim/
├── init.lua                  # Entry point: leader key + requires
├── ftplugin/
│   └── markdown.lua          # Markdown preview (<localleader>m / :Md → md in current window)
├── lsp/                      # Neovim 0.11 native LSP configs (auto-loaded)
│   ├── ts_ls.lua
│   ├── gopls.lua
│   └── jsonls.lua
└── lua/
    ├── config/
    │   ├── options.lua       # vim.o / vim.opt / vim.g settings
    │   ├── keymaps.lua       # General keymaps (plugin keymaps stay in specs)
    │   ├── autocmds.lua      # Autocommands + LspAttach
    │   ├── lazy.lua          # Bootstrap lazy.nvim + load plugins
    │   └── lsp.lua           # vim.lsp.config("*") + vim.lsp.enable()
    └── plugins/              # lazy.nvim auto-discovers all files here
        ├── theme.lua         # github-nvim-theme
        ├── treesitter.lua    # nvim-treesitter + treesitter-context
        ├── gitsigns.lua      # gitsigns
        ├── ui.lua            # mini.icons, snacks (picker, explorer, scroll, indent), satellite.nvim, lualine, scrollEOF
        ├── editing.lua       # which-key, hydra, multicursor.nvim, vim-sleuth, copilot.lua
        ├── completion.lua    # blink.cmp, blink-copilot, schemastore
        ├── conform.lua       # conform.nvim (format-on-save)
        └── jinja.lua         # Jinja2 (ftdetect only, no plugin)
```

## Load Order

```
init.lua
  1. vim.g.mapleader = " "        (must be before any plugin/keymap)
  2. require("config.options")     (vim.o/vim.opt settings)
  3. require("config.autocmds")    (autocommands, LspAttach)
  4. require("config.keymaps")     (general keymaps)
  5. require("config.lazy")        (bootstrap + load plugins)
  6. require("config.lsp")         (blink.cmp capabilities + vim.lsp.enable)
```

`mapleader` must be set before any keymaps or plugins reference it. `config.lsp` loads last because `vim.lsp.config("*")` needs `blink.cmp` (loaded by lazy.nvim) for LSP capabilities.

## How to Add a New Plugin

1. Create a new file in `lua/plugins/` (or add to an existing grouped file like `ui.lua`)
2. Return a lazy.nvim spec table: `return { "author/plugin-name", opts = { ... } }`
3. Plugin-specific keymaps go in the spec's `keys` field for lazy-loading
4. Restart nvim or run `:Lazy sync`

## How to Add a New LSP Server

1. Create `lsp/<server_name>.lua` returning `{ cmd = {...}, filetypes = {...}, root_markers = {...} }`
2. Add the server name to the `vim.lsp.enable()` list in `lua/config/lsp.lua`
3. Install the language server binary (e.g., via `brew` or `npm`)

## Key Design Choices

- **Plugin-specific keymaps stay in plugin specs** -- keeps keymaps co-located with the plugin config and enables lazy-loading
- **General keymaps in `lua/config/keymaps.lua`** -- vim motions, navigation, leader shortcuts not tied to any plugin
- **`lsp/` directory uses Neovim 0.11 native feature** -- no `nvim-lspconfig` plugin needed; Neovim auto-loads configs from `lsp/*.lua` when matching filetypes are opened

## Formatting

Format-on-save is handled by `conform.nvim` (`plugins/conform.lua`). Formatters per filetype are configured there.
