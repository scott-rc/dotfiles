vim.pack.add({
	"https://github.com/projekt0n/github-nvim-theme",
	{ src = "https://github.com/saghen/blink.cmp", version = vim.version.range("1.0") },
	"https://github.com/fang2hou/blink-copilot",
	"https://github.com/b0o/SchemaStore.nvim",
	"https://github.com/folke/which-key.nvim",
	"https://github.com/nvimtools/hydra.nvim",
	"https://github.com/jake-stewart/multicursor.nvim",
	"https://github.com/tpope/vim-sleuth",
	"https://github.com/zbirenbaum/copilot.lua",
	"https://github.com/lewis6991/gitsigns.nvim",
	"https://github.com/stevearc/conform.nvim",
	"https://github.com/nvim-treesitter/nvim-treesitter",
	"https://github.com/nvim-treesitter/nvim-treesitter-context",
	"https://github.com/echasnovski/mini.icons",
	"https://github.com/folke/snacks.nvim",
	"https://github.com/Aasim-A/scrollEOF.nvim",
	"https://github.com/lewis6991/satellite.nvim",
})

-- TSUpdate after treesitter install/update
vim.api.nvim_create_autocmd("PackChanged", {
	callback = function(ev)
		if ev.data.spec.name == "nvim-treesitter" and ev.data.kind ~= "delete" then
			vim.cmd("TSUpdate")
		end
	end,
})

-- Load plugins in dependency order
require("plugins.theme")
require("plugins.treesitter")
require("plugins.completion")
require("plugins.editing")
require("plugins.gitsigns")
require("plugins.conform")
require("plugins.ui")
