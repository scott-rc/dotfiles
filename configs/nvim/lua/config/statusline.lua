local M = {}

local mode_map = {
	n = { "NORMAL", "StatusLineModeNormal" },
	i = { "INSERT", "StatusLineModeInsert" },
	v = { "VISUAL", "StatusLineModeVisual" },
	V = { "V-LINE", "StatusLineModeVisual" },
	["\22"] = { "V-BLOCK", "StatusLineModeVisual" },
	c = { "COMMAND", "StatusLineModeCommand" },
	R = { "REPLACE", "StatusLineModeReplace" },
	t = { "TERMINAL", "StatusLineModeTerminal" },
}

function M.mode()
	local m = vim.fn.mode()
	local info = mode_map[m] or { m:upper(), "StatusLine" }
	return "%#" .. info[2] .. "# " .. info[1] .. " %*"
end

-- Define highlight groups (called from theme.lua apply_theme)
function M.set_highlights()
	vim.api.nvim_set_hl(0, "StatusLineModeNormal", { fg = "#e6edf3", bg = "#238636", bold = true })
	vim.api.nvim_set_hl(0, "StatusLineModeInsert", { fg = "#e6edf3", bg = "#1f6feb", bold = true })
	vim.api.nvim_set_hl(0, "StatusLineModeVisual", { fg = "#e6edf3", bg = "#8957e5", bold = true })
	vim.api.nvim_set_hl(0, "StatusLineModeCommand", { fg = "#e6edf3", bg = "#d29922", bold = true })
	vim.api.nvim_set_hl(0, "StatusLineModeReplace", { fg = "#e6edf3", bg = "#da3633", bold = true })
	vim.api.nvim_set_hl(0, "StatusLineModeTerminal", { fg = "#e6edf3", bg = "#238636", bold = true })
end

return M
