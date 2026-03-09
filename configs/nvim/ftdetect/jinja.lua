-- Filetype detection for Jinja2 templates
vim.filetype.add({
	pattern = {
		[".*%.yaml%.j2"] = "yaml.jinja",
		[".*%.yml%.j2"] = "yaml.jinja",
	},
})

-- Jinja2 highlight groups
vim.api.nvim_set_hl(0, "JinjaVariable", { link = "PreProc" })
vim.api.nvim_set_hl(0, "JinjaTag", { link = "Keyword" })
vim.api.nvim_set_hl(0, "JinjaComment", { link = "Comment" })

-- Use Vim regex syntax for yaml.jinja (tree-sitter yaml can't parse jinja directives)
vim.api.nvim_create_autocmd("FileType", {
	pattern = "yaml.jinja",
	callback = function(args)
		-- Defer so this runs after treesitter.lua's FileType autocmd
		vim.schedule(function()
			if not vim.api.nvim_buf_is_valid(args.buf) then
				return
			end
			-- Stop tree-sitter (it can't parse jinja syntax mixed into yaml)
			vim.treesitter.stop(args.buf)
			-- Enable Vim's built-in yaml syntax highlighting
			vim.bo[args.buf].syntax = "yaml"
			-- Overlay jinja delimiter highlighting (matchadd is window-local)
			vim.api.nvim_buf_call(args.buf, function()
				vim.fn.matchadd("JinjaVariable", "{{.\\{-}}}")
				vim.fn.matchadd("JinjaTag", "{%.\\{-}%}")
				vim.fn.matchadd("JinjaComment", "{#.\\{-}#}")
			end)
		end)
	end,
})
