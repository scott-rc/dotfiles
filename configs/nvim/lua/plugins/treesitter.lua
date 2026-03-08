return {
	{
		"nvim-treesitter/nvim-treesitter",
		build = ":TSUpdate",
		config = function()
			local wanted = {
				"bash",
				"css",
				"diff",
				"fish",
				"go",
				"gomod",
				"graphql",
				"html",
				"javascript",
				"json",
				"lua",
				"markdown",
				"markdown_inline",
				"ruby",
				"rust",
				"toml",
				"tsx",
				"typescript",
				"vim",
				"vimdoc",
				"yaml",
			}
			local installed = require("nvim-treesitter.config").get_installed()
			local missing = vim.tbl_filter(function(lang)
				return not vim.list_contains(installed, lang)
			end, wanted)
			if #missing > 0 then
				require("nvim-treesitter.install").install(missing)
			end

			vim.api.nvim_create_autocmd("FileType", {
				callback = function(args)
					if pcall(vim.treesitter.start, args.buf) then
						vim.bo[args.buf].indentexpr = "v:lua.require'nvim-treesitter'.indentexpr()"
					end
				end,
			})
		end,
	},

	-- Sticky scroll (shows containing function/class at top)
	{
		"nvim-treesitter/nvim-treesitter-context",
		dependencies = { "nvim-treesitter/nvim-treesitter" },
		opts = {
			max_lines = 6,
		},
	},
}
