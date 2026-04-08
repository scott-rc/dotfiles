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

	-- Completion menu: consistent background, transparent borders, subtle selection
	local editor_bg = vim.api.nvim_get_hl(0, { name = "Normal", link = false }).bg
	vim.api.nvim_set_hl(0, "BlinkCmpMenu", { bg = editor_bg })
	vim.api.nvim_set_hl(0, "BlinkCmpMenuBorder", { fg = "#565f89", bg = editor_bg })
	vim.api.nvim_set_hl(0, "BlinkCmpMenuSelection", { bg = "#1c3d6a" })
	vim.api.nvim_set_hl(0, "BlinkCmpDoc", { bg = editor_bg })
	vim.api.nvim_set_hl(0, "BlinkCmpDocBorder", { fg = "#565f89", bg = editor_bg })
	vim.api.nvim_set_hl(0, "BlinkCmpSignatureHelp", { bg = editor_bg })
	vim.api.nvim_set_hl(0, "BlinkCmpSignatureHelpBorder", { fg = "#565f89", bg = editor_bg })

	-- Pmenu highlight groups
	vim.api.nvim_set_hl(0, "Pmenu", { bg = editor_bg })
	vim.api.nvim_set_hl(0, "PmenuSel", { bg = "#1c3d6a" })
	vim.api.nvim_set_hl(0, "PmenuBorder", { fg = "#565f89", bg = editor_bg })

	-- Statusline mode highlights
	require("config.statusline").set_highlights()
end

apply_theme()

-- Re-check on focus so theme follows system appearance changes
vim.api.nvim_create_autocmd("FocusGained", {
	callback = apply_theme,
})
