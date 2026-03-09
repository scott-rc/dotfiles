-- Filetype detection for Jinja2 templates
vim.filetype.add({
	pattern = {
		[".*%.yaml%.j2"] = "yaml.jinja",
		[".*%.yml%.j2"] = "yaml.jinja",
	},
})

-- Jinja2 highlight groups
vim.api.nvim_set_hl(0, "JinjaDelimiter", { link = "Operator" })
vim.api.nvim_set_hl(0, "JinjaKeyword", { link = "Keyword" })
vim.api.nvim_set_hl(0, "JinjaIdentifier", { link = "Identifier" })
vim.api.nvim_set_hl(0, "JinjaFilter", { link = "Function" })
vim.api.nvim_set_hl(0, "JinjaString", { link = "String" })
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
			-- Overlay jinja highlighting via matchadd (separate layer, no syntax conflicts)
			vim.api.nvim_buf_call(args.buf, function()
				-- Comments: {# ... #} (highest priority — covers entire block)
				vim.fn.matchadd("JinjaComment", "{#.\\{-}#}")

				-- Delimiters: {{ }} {% %} {# #}
				vim.fn.matchadd("JinjaDelimiter", "{{\\|}}\\|{%-\\?\\|-\\?%}\\|{#\\|#}")

				-- Keywords inside {% %} tags
				vim.fn.matchadd(
					"JinjaKeyword",
					"{%-\\?\\s*\\zs\\(for\\|endfor\\|if\\|elif\\|else\\|endif\\|set\\|endset\\|block\\|endblock\\|extends\\|include\\|import\\|from\\|macro\\|endmacro\\|call\\|endcall\\|filter\\|endfilter\\|raw\\|endraw\\|in\\|not\\|and\\|or\\|is\\|recursive\\)\\>\\ze"
				)

				-- Filters: word after |  (inside both {{ }} and {% %})
				vim.fn.matchadd("JinjaFilter", "\\({{\\|{%\\)\\_.\\{-}\\zs|\\s*\\w\\+\\ze")

				-- Strings inside jinja blocks
				vim.fn.matchadd("JinjaString", "\\({{\\|{%\\)\\_.\\{-}\\zs'[^']*'\\ze")
				vim.fn.matchadd("JinjaString", '\\({{\\|{%\\)\\_.\\{-}\\zs"[^"]*"\\ze')

				-- Identifiers (variable names) inside {{ }}
				vim.fn.matchadd("JinjaIdentifier", "{{-\\?\\s*\\zs\\w\\+\\ze")
				-- Identifiers after keywords in {% %}
				vim.fn.matchadd(
					"JinjaIdentifier",
					"{%-\\?\\s*\\(for\\|if\\|elif\\|set\\|block\\|macro\\|from\\|import\\|include\\|extends\\)\\s\\+\\zs\\w\\+\\ze"
				)
			end)
		end)
	end,
})
