local augroup = vim.api.nvim_create_augroup("user_config", { clear = true })

vim.api.nvim_create_autocmd({ "FocusGained", "BufEnter", "CursorHold", "CursorHoldI" }, {
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
			buf = args.buf,
			desc = "Go to definition / references",
		})
		vim.keymap.set("n", "gd", function() Snacks.picker.lsp_definitions() end, {
			buf = args.buf,
			desc = "Go to definition",
		})
		vim.keymap.set("n", "grr", function()
			Snacks.picker.lsp_references()
		end, {
			buf = args.buf,
			desc = "Find references",
		})
		vim.keymap.set("n", "<C-Space>", function()
			local bufnr = vim.api.nvim_get_current_buf()
			local diagnostics = vim.diagnostic.get(bufnr, { lnum = vim.fn.line(".") - 1 })
			local clients = vim.lsp.get_clients({ bufnr = bufnr, method = "textDocument/hover" })

			local function render(hover_lines)
				local lines = {}
				for _, d in ipairs(diagnostics) do
					local sev = vim.diagnostic.severity[d.severity]
					sev = sev:sub(1, 1) .. sev:sub(2):lower()
					local header = "**" .. sev .. "**"
					if d.source and d.source ~= "" then
						header = header .. " *(" .. d.source .. ")*"
					end
					table.insert(lines, header)
					for _, msg in ipairs(vim.split(d.message, "\n", { plain = true })) do
						table.insert(lines, msg)
					end
					table.insert(lines, "")
				end
				if hover_lines and #hover_lines > 0 then
					if #lines > 0 then
						table.insert(lines, "---")
						table.insert(lines, "")
					end
					vim.list_extend(lines, hover_lines)
				end
				if #lines == 0 then
					return
				end
				local _, winid = vim.lsp.util.open_floating_preview(lines, "markdown", {
					border = "rounded",
					focusable = true,
					focus_id = "hover_with_diagnostics",
				})
				if winid and vim.api.nvim_win_is_valid(winid) then
					vim.wo[winid].concealcursor = "nvic"
				end
			end

			if #clients == 0 then
				render(nil)
				return
			end

			local params = vim.lsp.util.make_position_params(0, clients[1].offset_encoding)
			vim.lsp.buf_request_all(bufnr, "textDocument/hover", params, function(results)
				local hover_lines = {}
				for _, res in pairs(results) do
					if res.result and res.result.contents then
						local md = vim.lsp.util.convert_input_to_markdown_lines(res.result.contents)
						vim.list_extend(hover_lines, md)
					end
				end
				render(hover_lines)
			end)
		end, {
			buf = args.buf,
			desc = "Hover documentation + diagnostics",
		})
		vim.keymap.set({ "n", "v", "i" }, "<D-.>", vim.lsp.buf.code_action, {
			buf = args.buf,
			desc = "Code actions",
		})
		vim.keymap.set({ "n", "v", "i" }, "<M-CR>", vim.lsp.buf.code_action, {
			buf = args.buf,
			desc = "Code actions",
		})
		vim.keymap.set("n", "<M-r>", vim.lsp.buf.rename, {
			buf = args.buf,
			desc = "Rename symbol",
		})
	end,
})
