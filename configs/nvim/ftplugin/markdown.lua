local function md()
	local file = vim.api.nvim_buf_get_name(0)
	if file == "" then
		vim.notify("No file to open", vim.log.levels.WARN)
		return
	end
	vim.cmd("enew")
	local buf = vim.api.nvim_get_current_buf()
	vim.fn.jobstart("md " .. vim.fn.shellescape(file), {
		term = true,
		on_exit = function()
			if vim.api.nvim_buf_is_valid(buf) then
				vim.api.nvim_buf_delete(buf, { force = true })
			end
		end,
	})
	vim.cmd("startinsert")
end

vim.api.nvim_buf_create_user_command(0, "Md", md, { desc = "Open with md" })
vim.keymap.set("n", "<localleader>m", md, { buffer = 0, desc = "Open with md" })
