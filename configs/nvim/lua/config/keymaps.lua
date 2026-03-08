-- Literal search (no regex)
vim.keymap.set("n", "/", "/\\V", { desc = "Literal search (no regex)" })

-- Escape
vim.keymap.set({ "n", "v", "i" }, "fd", "<Esc>", { desc = "Escape" })
vim.keymap.set("n", "<Esc>", "<cmd>nohlsearch<CR>", { desc = "Clear search highlight" })

-- Option+Delete word deletion (Ghostty sends Alt/ESC prefix for Option key)
vim.keymap.set("i", "<M-BS>", "<C-w>", { desc = "Delete word backward" })
-- Cmd+Backspace: Ghostty sends \x15 (Ctrl-U) which already does delete-to-beginning in insert mode

-- Move lines with Option+j/k
vim.keymap.set("n", "<M-j>", "<cmd>m .+1<CR>==", { desc = "Move line down" })
vim.keymap.set("n", "<M-k>", "<cmd>m .-2<CR>==", { desc = "Move line up" })
vim.keymap.set("v", "<M-j>", ":m '>+1<CR>gv=gv", { desc = "Move selection down" })
vim.keymap.set("v", "<M-k>", ":m '<-2<CR>gv=gv", { desc = "Move selection up" })

-- Navigate by display lines when no count is given (for wrapped lines)
vim.keymap.set({ "n", "v" }, "j", function()
	return vim.v.count == 0 and "gj" or "j"
end, { expr = true, desc = "Down (wrap-aware)" })
vim.keymap.set({ "n", "v" }, "k", function()
	return vim.v.count == 0 and "gk" or "k"
end, { expr = true, desc = "Up (wrap-aware)" })

-- Navigation
vim.keymap.set({ "n", "v" }, "<Space>", "<Nop>")
vim.keymap.set({ "n", "v" }, ";", ":", { desc = "Command mode" })
vim.keymap.set({ "n", "v" }, "<S-h>", "^", { desc = "Start of line" })
vim.keymap.set({ "n", "v" }, "<S-l>", "$", { desc = "End of line" })

-- Window navigation
vim.keymap.set("n", "<C-h>", "<C-w>h", { desc = "Focus left window" })
vim.keymap.set("n", "<C-j>", "<C-w>j", { desc = "Focus below window" })
vim.keymap.set("n", "<C-k>", "<C-w>k", { desc = "Focus above window" })
vim.keymap.set("n", "<C-l>", "<C-w>l", { desc = "Focus right window" })

-- Leader
vim.keymap.set({ "n", "v" }, "<leader>w", "<cmd>q<CR>", { desc = "Close" })
vim.keymap.set({ "n", "v" }, "<leader>q", "<cmd>qa<CR>", { desc = "Quit all" })
vim.keymap.set({ "n", "v" }, "<leader>s", "<cmd>w<CR>", { desc = "Save" })
vim.keymap.set({ "n", "v" }, "<leader>S", "<cmd>noautocmd w<CR>", { desc = "Save (no format)" })
vim.keymap.set("n", "<leader>t", "<cmd>tabnew<CR>", { desc = "New tab" })
vim.keymap.set("n", "<leader>[", "<cmd>tabp<CR>", { desc = "Previous tab" })
vim.keymap.set("n", "<leader>]", "<cmd>tabn<CR>", { desc = "Next tab" })
vim.keymap.set("n", "<leader>%", "<cmd>source %<CR>", { desc = "Source file" })
vim.keymap.set("v", "<D-c>", '"+y', { desc = "Copy to clipboard" })
vim.keymap.set({ "n", "v", "i" }, "<D-q>", "<cmd>qa<CR>", { desc = "Quit" })
vim.keymap.set({ "n", "v", "i" }, "<D-s>", "<cmd>w<CR>", { desc = "Save" })
vim.keymap.set({ "n", "v", "i" }, "<D-a>", "<Esc>ggVG", { desc = "Select all" })
vim.keymap.set("n", "<leader>/", "gcc", { remap = true, desc = "Toggle comment" })
vim.keymap.set("v", "<leader>/", "gc", { remap = true, desc = "Toggle comment" })
vim.keymap.set("n", "<D-/>", "gcc", { remap = true, desc = "Toggle comment" })
vim.keymap.set("v", "<D-/>", "gc", { remap = true, desc = "Toggle comment" })
vim.keymap.set("i", "<D-/>", "<Esc>gcc", { remap = true, desc = "Toggle comment" })

-- Stay in visual mode after indent
vim.keymap.set("v", "<", "<gv", { desc = "Outdent and reselect" })
vim.keymap.set("v", ">", ">gv", { desc = "Indent and reselect" })

-- Diagnostic navigation (error-only; ]d/[d for all severities is built-in)
vim.keymap.set("n", "]D", function()
	vim.diagnostic.jump({ count = 1, severity = vim.diagnostic.severity.ERROR })
end, { desc = "Next error" })
vim.keymap.set("n", "[D", function()
	vim.diagnostic.jump({ count = -1, severity = vim.diagnostic.severity.ERROR })
end, { desc = "Prev error" })
vim.keymap.set({ "n", "v", "i" }, "<D-z>", "<cmd>undo<CR>", { desc = "Undo" })
vim.keymap.set({ "n", "v", "i" }, "<D-S-z>", "<cmd>redo<CR>", { desc = "Redo" })
vim.keymap.set({ "n", "v", "i" }, "<D-w>", "<cmd>bdelete<CR>", { desc = "Close buffer" })
vim.keymap.set({ "n", "v" }, "<D-[>", "<C-o>", { desc = "Go back" })
vim.keymap.set({ "n", "v" }, "<D-]>", "<C-i>", { desc = "Go forward" })
vim.keymap.set("i", "<D-[>", "<Esc><C-o>", { desc = "Go back" })
vim.keymap.set("i", "<D-]>", "<Esc><C-i>", { desc = "Go forward" })

-- Focus primary buffer (D-1)
local function is_file_win(w)
	local buf = vim.api.nvim_win_get_buf(w)
	return vim.bo[buf].buftype == "" and vim.bo[buf].filetype ~= "neo-tree"
end

vim.keymap.set({ "n", "v", "i" }, "<D-1>", function()
	local win = vim.g._last_file_win
	if win and vim.api.nvim_win_is_valid(win) and is_file_win(win) then
		vim.api.nvim_set_current_win(win)
		return
	end
	for _, w in ipairs(vim.api.nvim_list_wins()) do
		if is_file_win(w) then
			vim.api.nvim_set_current_win(w)
			return
		end
	end
end, { desc = "Focus primary buffer" })

-- Yank paths
vim.keymap.set({ "n", "v" }, "<leader>yp", function()
	vim.fn.setreg("+", vim.fn.fnamemodify(vim.fn.expand("%"), ":."))
end, { desc = "Copy relative path" })
vim.keymap.set({ "n", "v" }, "<leader>yP", function()
	vim.fn.setreg("+", vim.fn.expand("%:p"))
end, { desc = "Copy absolute path" })
vim.keymap.set("n", "<leader>yl", function()
	vim.fn.setreg("+", vim.fn.fnamemodify(vim.fn.expand("%"), ":.") .. ":" .. vim.fn.line("."))
end, { desc = "Copy relative path:line" })
vim.keymap.set("n", "<leader>yL", function()
	vim.fn.setreg("+", vim.fn.expand("%:p") .. ":" .. vim.fn.line("."))
end, { desc = "Copy absolute path:line" })
vim.keymap.set("v", "<leader>yl", function()
	local s, e = vim.fn.line("v"), vim.fn.line(".")
	if s > e then
		s, e = e, s
	end
	vim.fn.setreg("+", vim.fn.fnamemodify(vim.fn.expand("%"), ":.") .. ":" .. s .. "-" .. e)
end, { desc = "Copy relative path:lines" })
vim.keymap.set("v", "<leader>yL", function()
	local s, e = vim.fn.line("v"), vim.fn.line(".")
	if s > e then
		s, e = e, s
	end
	vim.fn.setreg("+", vim.fn.expand("%:p") .. ":" .. s .. "-" .. e)
end, { desc = "Copy absolute path:lines" })

-- GitHub URLs
local function github_url(opts)
	local remote = vim.fn.systemlist("git remote get-url origin")[1]
	if vim.v.shell_error ~= 0 or not remote then
		vim.notify("No origin remote", vim.log.levels.WARN)
		return nil
	end
	remote = remote:gsub("git@github%.com:", "https://github.com/"):gsub("%.git$", "")
	local branch = opts.branch or vim.fn.systemlist("git rev-parse --abbrev-ref HEAD")[1]
	local git_root = vim.fn.systemlist("git rev-parse --show-toplevel")[1]
	local rel_path = vim.fn.expand("%:p"):sub(#git_root + 2)
	local url = remote .. "/blob/" .. branch .. "/" .. rel_path
	if opts.visual then
		local s, e = vim.fn.line("v"), vim.fn.line(".")
		if s > e then
			s, e = e, s
		end
		url = url .. "#L" .. s .. "-L" .. e
	end
	return url
end

vim.keymap.set("n", "<leader>go", function()
	local url = github_url({})
	if url then
		vim.fn.system({ "open", url })
	end
end, { desc = "Open in GitHub" })
vim.keymap.set("v", "<leader>go", function()
	local url = github_url({ visual = true })
	if url then
		vim.fn.system({ "open", url })
	end
end, { desc = "Open in GitHub (selection)" })
vim.keymap.set("n", "<leader>gO", function()
	local url = github_url({ branch = "main" })
	if url then
		vim.fn.system({ "open", url })
	end
end, { desc = "Open in GitHub (main)" })
vim.keymap.set("v", "<leader>gO", function()
	local url = github_url({ branch = "main", visual = true })
	if url then
		vim.fn.system({ "open", url })
	end
end, { desc = "Open in GitHub (main, selection)" })

-- Wildmenu navigation
vim.o.wildcharm = vim.fn.char2nr(vim.api.nvim_replace_termcodes("<C-z>", true, true, true))
vim.keymap.set("c", "<up>", function()
	return vim.fn.wildmenumode() == 1 and "<left>" or "<up>"
end, { expr = true, desc = "Wildmenu: previous match" })
vim.keymap.set("c", "<down>", function()
	return vim.fn.wildmenumode() == 1 and "<right>" or "<down>"
end, { expr = true, desc = "Wildmenu: next match" })
vim.keymap.set("c", "<left>", function()
	return vim.fn.wildmenumode() == 1 and "<up>" or "<left>"
end, { expr = true, desc = "Wildmenu: parent dir" })
vim.keymap.set("c", "<right>", function()
	return vim.fn.wildmenumode() == 1 and " <bs><C-z>" or "<right>"
end, { expr = true, desc = "Wildmenu: enter dir" })
