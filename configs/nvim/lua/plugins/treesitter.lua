local wanted = {
	"bash",
	"css",
	"cue",
	"diff",
	"fish",
	"go",
	"gomod",
	"graphql",
	"html",
	"javascript",
	"json",
	"ruby",
	"rust",
	"toml",
	"tsx",
	"typescript",
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

-- Sticky scroll (shows containing function/class at top)
require("treesitter-context").setup({
	max_lines = 6,
})
