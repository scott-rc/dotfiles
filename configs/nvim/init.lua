vim.g.loaded_netrw = 1
vim.g.loaded_netrwPlugin = 1
vim.g.mapleader = " "

require("vim._core.ui2").enable({})

require("config.options")
require("config.autocmds")
require("config.keymaps")
require("config.pack")
require("config.lsp")
