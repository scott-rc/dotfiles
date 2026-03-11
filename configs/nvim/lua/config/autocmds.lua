local augroup = vim.api.nvim_create_augroup("user_config", { clear = true })

vim.api.nvim_create_autocmd({ "FocusGained", "BufEnter" }, {
	group = augroup,
	command = "checktime",
})

-- Track the last window showing a regular file buffer
vim.api.nvim_create_autocmd("BufEnter", {
	group = augroup,
	callback = function()
		if vim.bo.buftype == "" and vim.bo.filetype ~= "snacks_picker_list" then
			vim.g._last_file_win = vim.api.nvim_get_current_win()
		end
	end,
})

-- gd, grn, gra, grr, K are Neovim 0.11+ built-in LSP defaults
vim.api.nvim_create_autocmd("LspAttach", {
	group = augroup,
	callback = function(args)
		vim.keymap.set({ "n", "v", "i" }, "<D-b>", function()
			local client = vim.lsp.get_clients({ bufnr = 0 })[1]
			local params = vim.lsp.util.make_position_params(0, client and client.offset_encoding or "utf-16")
			vim.lsp.buf_request(0, "textDocument/definition", params, function(err, result)
				if err then
					return
				end
				local defs = result or {}
				if not vim.islist(defs) then
					defs = { defs }
				end
				local cur_uri = vim.uri_from_bufnr(0)
				local cur_line = params.position.line
				local cur_col = params.position.character
				local at_def = false
				for _, def in ipairs(defs) do
					local loc = def.targetSelectionRange or def.targetRange or def.range
					local uri = def.targetUri or def.uri or ""
					if
						uri == cur_uri
						and loc
						and loc.start.line == cur_line
						and loc.start.character <= cur_col
						and (loc["end"].character >= cur_col or loc["end"].line > cur_line)
					then
						at_def = true
						break
					end
				end
				vim.schedule(function()
					if at_def or #defs == 0 then
						Snacks.picker.lsp_references()
					else
						Snacks.picker.lsp_definitions()
					end
				end)
			end)
		end, {
			buffer = args.buf,
			desc = "Go to definition / references",
		})
		vim.keymap.set("n", "gd", function() Snacks.picker.lsp_definitions() end, {
			buffer = args.buf,
			desc = "Go to definition",
		})
		vim.keymap.set("n", "grr", function()
			Snacks.picker.lsp_references()
		end, {
			buffer = args.buf,
			desc = "Find references",
		})
		vim.keymap.set("n", "<C-Space>", vim.lsp.buf.hover, {
			buffer = args.buf,
			desc = "Hover documentation",
		})
		vim.keymap.set({ "n", "v", "i" }, "<D-.>", vim.lsp.buf.code_action, {
			buffer = args.buf,
			desc = "Code actions",
		})
		vim.keymap.set({ "n", "v", "i" }, "<M-CR>", vim.lsp.buf.code_action, {
			buffer = args.buf,
			desc = "Code actions",
		})
		vim.keymap.set("n", "<M-r>", vim.lsp.buf.rename, {
			buffer = args.buf,
			desc = "Rename symbol",
		})
	end,
})
