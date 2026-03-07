return {
	"projekt0n/github-nvim-theme",
	lazy = false,
	priority = 1000,
	config = function()
		require("github-theme").setup({
			options = {
				styles = {
					comments = "italic",
				},
			},
			groups = {
				all = {
					["@keyword"] = { link = "Keyword" },
					["@keyword.import"] = { link = "Include" },
					["@keyword.coroutine"] = { link = "Keyword" },
				},
			},
		})

		local function apply_theme()
			local target = "github_dark_default"
			if vim.g.colors_name ~= target then
				vim.cmd.colorscheme(target)
			end
			-- Underline function/method calls while preserving the theme's Function color
			local fn_hl = vim.api.nvim_get_hl(0, { name = "Function", link = false })
			fn_hl.underline = true
			vim.api.nvim_set_hl(0, "@function.call", fn_hl)
			vim.api.nvim_set_hl(0, "@method.call", fn_hl)
		end

		apply_theme()

		-- Re-check on focus so theme follows system appearance changes
		vim.api.nvim_create_autocmd("FocusGained", {
			callback = apply_theme,
		})
	end,
}
