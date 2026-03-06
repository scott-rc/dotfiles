-- ============================================================================
-- Settings
-- ============================================================================

vim.g.loaded_netrw = 1
vim.g.loaded_netrwPlugin = 1
vim.g.mapleader = " "
vim.g.vim_json_conceal = 0
vim.o.timeoutlen = 300

-- Search
vim.o.ignorecase = true
vim.o.smartcase = true
vim.o.gdefault = true
vim.keymap.set("n", "/", "/\\V", { desc = "Literal search (no regex)" })

-- Splits
vim.o.splitbelow = true
vim.o.splitright = true
vim.o.splitkeep = "screen"

-- Display
vim.o.title = true
vim.o.titlestring = " %f%( %m%)"
vim.o.cmdheight = 0
vim.o.showmode = false
vim.o.laststatus = 3
vim.o.signcolumn = "yes"
vim.o.showmatch = true
vim.o.number = true
vim.o.cursorline = true
vim.o.wrap = false
vim.o.scrolloff = 999
vim.opt.listchars = { tab = "→ ", trail = "·", nbsp = "+", extends = ">", precedes = "<" }

-- Indentation
vim.o.expandtab = true
vim.o.tabstop = 4
vim.o.shiftwidth = 4
vim.o.smartindent = true
vim.opt.formatoptions:append("o")

-- Folding (treesitter-based, all open by default)
vim.o.foldmethod = "expr"
vim.o.foldexpr = "v:lua.vim.treesitter.foldexpr()"
vim.o.foldlevelstart = 99
vim.o.foldtext = ""
vim.o.fillchars = "fold: ,foldopen:▼,foldclose:▶,foldsep:│"

-- Behavior
vim.o.mouse = "n"
vim.o.updatetime = 250
vim.opt.whichwrap:append("<,>,h,l")

-- Persistence
vim.o.undofile = true
vim.o.writebackup = false
vim.o.swapfile = false

-- ============================================================================
-- Keybindings
-- ============================================================================

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

-- ============================================================================
-- Autocommands
-- ============================================================================

local augroup = vim.api.nvim_create_augroup("user_config", { clear = true })

local _neotree_width = nil

local function toggle_neotree_files()
	local manager = require("neo-tree.sources.manager")
	local state = manager.get_state("filesystem")
	local neo_win = state.winid
	if neo_win and vim.api.nvim_win_is_valid(neo_win) then
		if vim.api.nvim_get_current_win() == neo_win then
			_neotree_width = vim.api.nvim_win_get_width(neo_win)
			local ei = vim.o.eventignore
			vim.o.eventignore = "BufEnter,WinEnter,WinLeave,BufLeave"
			manager.close("filesystem")
			vim.o.eventignore = ei
		else
			vim.api.nvim_set_current_win(neo_win)
		end
	else
		vim.cmd("Neotree focus source=filesystem")
		if _neotree_width then
			local s = manager.get_state("filesystem")
			if s.winid and vim.api.nvim_win_is_valid(s.winid) then
				vim.api.nvim_win_set_width(s.winid, _neotree_width)
			end
		end
	end
end

-- When opened with a directory arg (e.g. `nvim .`), replace the directory
-- buffer with an empty buffer and open neo-tree as a proper sidebar
vim.api.nvim_create_autocmd("VimEnter", {
	group = augroup,
	callback = function()
		if vim.fn.argc() == 1 and vim.fn.isdirectory(vim.fn.argv(0)) == 1 then
			vim.schedule(function()
				vim.cmd("bdelete")
				require("neo-tree")
				vim.cmd("Neotree source=filesystem")
			end)
		end
	end,
})

vim.api.nvim_create_autocmd({ "FocusGained", "BufEnter" }, {
	group = augroup,
	command = "checktime",
})

-- Track the last window showing a regular file buffer
vim.api.nvim_create_autocmd("BufEnter", {
	group = augroup,
	callback = function()
		if vim.bo.buftype == "" and vim.bo.filetype ~= "neo-tree" then
			vim.g._last_file_win = vim.api.nvim_get_current_win()
		end
	end,
})

-- Lock neo-tree sidebar width so equalalways doesn't resize it during focus changes
vim.api.nvim_create_autocmd("FileType", {
	group = augroup,
	pattern = "neo-tree",
	callback = function()
		vim.wo.winfixwidth = true
	end,
})

-- ============================================================================
-- Bootstrap lazy.nvim
-- ============================================================================

local lazypath = vim.fn.stdpath("data") .. "/lazy/lazy.nvim"
if not vim.uv.fs_stat(lazypath) then
	vim.fn.system({
		"git",
		"clone",
		"--filter=blob:none",
		"https://github.com/folke/lazy.nvim.git",
		"--branch=stable",
		lazypath,
	})
end
vim.opt.rtp:prepend(lazypath)

-- ============================================================================
-- Plugins
-- ============================================================================

require("lazy").setup({
	-- Theme
	{
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

			local function is_dark_mode()
				local result = vim.fn.system("defaults read -g AppleInterfaceStyle 2>/dev/null")
				return result:match("Dark") ~= nil
			end

			local function apply_theme()
				local target = is_dark_mode() and "github_dark_default" or "github_dark_dimmed"
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
	},

	-- Treesitter (syntax highlighting)
	{
		"nvim-treesitter/nvim-treesitter",
		build = ":TSUpdate",
		config = function()
			local wanted = {
				"bash",
				"css",
				"diff",
				"fish",
				"go",
				"gomod",
				"graphql",
				"html",
				"javascript",
				"json",
				"lua",
				"markdown",
				"markdown_inline",
				"ruby",
				"rust",
				"toml",
				"tsx",
				"typescript",
				"vim",
				"vimdoc",
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
		end,
	},

	-- Keybinding popup
	{
		"folke/which-key.nvim",
		event = "VeryLazy",
		opts = {
			preset = "modern",
			triggers = {
				{ "<auto>", mode = "nvsot" },
				{ "<leader>", mode = "v" },
			},
			spec = {
				{ "<leader>y", group = "Yank" },

				{ "<leader>g", group = "Git" },
				{ "<leader>l", group = "LSP" },

				{ "<C-w>r", group = "Resize" },
				{ "<leader>o", group = "Options" },
				{
					"<leader>ow",
					function()
						local wo = vim.wo
						wo.wrap = not wo.wrap
						wo.linebreak = wo.wrap
						wo.breakindent = wo.wrap
						wo.colorcolumn = wo.wrap and "100" or ""
					end,
					desc = "Toggle wrap",
				},
				{ "<leader>on", "<cmd>set number!<cr>", desc = "Toggle line numbers" },
				{ "<leader>or", "<cmd>set relativenumber!<cr>", desc = "Toggle relative numbers" },
				{ "<leader>oh", "<cmd>nohlsearch<cr>", desc = "Clear search highlight" },
				{ "<leader>os", "<cmd>set spell!<cr>", desc = "Toggle spell check" },
				{ "<leader>ol", "<cmd>set list!<cr>", desc = "Toggle invisible chars" },
				{ "<leader>oi", "<cmd>set ignorecase!<cr>", desc = "Toggle ignore case" },
			},
		},
	},

	-- Icons (file + folder-name-specific, e.g. src, test, node_modules)
	{
		"echasnovski/mini.icons",
		opts = {},
		config = function(_, opts)
			require("mini.icons").setup(opts)
			MiniIcons.mock_nvim_web_devicons()
		end,
	},

	-- File explorer
	{
		"nvim-neo-tree/neo-tree.nvim",
		branch = "v3.x",
		dependencies = {
			"nvim-lua/plenary.nvim",
			"MunifTanjim/nui.nvim",
			"echasnovski/mini.icons",
		},
		config = function()
			-- Monkey-patch: neo-tree crashes on deleted files whose parent directory
			-- was also deleted (fs_lstat returns nil → type "unknown" → no children
			-- table → table.insert crashes). Force non-existent extensionless paths
			-- to type "directory" so parent nodes always have a children table.
			local file_items = require("neo-tree.sources.common.file-items")
			local orig_create_item = file_items.create_item
			file_items.create_item = function(context, path, _type)
				if _type == nil and not vim.uv.fs_lstat(path) then
					local basename = vim.fn.fnamemodify(path, ":t")
					if not basename:match("%.") then
						_type = "directory"
					end
				end
				return orig_create_item(context, path, _type)
			end

			require("neo-tree").setup({
				log_level = "warn",
				default_component_configs = {
					icon = {
						provider = function(icon, node)
							local mini = require("mini.icons")
							local category = node.type == "directory" and "directory" or "file"
							local text, hl = mini.get(category, node.name)
							icon.text = text
							icon.highlight = hl
						end,
					},
				},
				commands = {
					open_and_refocus = function(state)
						local node = state.tree:get_node()
						if node.type == "directory" then
							require("neo-tree.sources." .. state.name .. ".commands").open(state)
							return
						end
						local ei = vim.o.eventignore
						vim.o.eventignore = "BufEnter,WinEnter,WinLeave,BufLeave"
						require("neo-tree.sources." .. state.name .. ".commands").open(state)
						vim.api.nvim_set_current_win(state.winid)
						vim.o.eventignore = ei
					end,
				},
				window = {
					position = "right",
					mappings = {
						["<cr>"] = "open_and_refocus",
						["<space>"] = "open_and_refocus",
						["<bs>"] = "delete",
						["<C-r>"] = "none",
					},
				},
				filesystem = {
					hijack_netrw_behavior = "disabled",
					follow_current_file = { enabled = true },
					use_libuv_file_watcher = true,
					filtered_items = {
						visible = true,
					},
				},
			})
		end,
		keys = {
			{ "<leader>e", toggle_neotree_files, desc = "File explorer" },
			{ "<C-e>", toggle_neotree_files, desc = "Focus/toggle file explorer" },
			{ "<D-e>", toggle_neotree_files, mode = { "n", "v", "i" }, desc = "Focus/toggle file explorer" },
		},
	},

	-- Git change indicators
	{
		"lewis6991/gitsigns.nvim",
		opts = {
			on_attach = function(bufnr)
				local gs = require("gitsigns")

				local function map(mode, l, r, desc)
					vim.keymap.set(mode, l, r, { buffer = bufnr, desc = desc })
				end

				-- Hunk navigation
				map("n", "]c", function()
					gs.nav_hunk("next")
				end, "Next hunk")
				map("n", "[c", function()
					gs.nav_hunk("prev")
				end, "Prev hunk")
				map("n", "]C", function()
					gs.nav_hunk("last")
				end, "Last hunk")
				map("n", "[C", function()
					gs.nav_hunk("first")
				end, "First hunk")

				-- Stage/unstage
				map("n", "<leader>gs", gs.stage_hunk, "Stage hunk")
				map("v", "<leader>gs", function()
					gs.stage_hunk({ vim.fn.line("."), vim.fn.line("v") })
				end, "Stage selected lines")
				map("n", "<leader>gu", gs.undo_stage_hunk, "Undo stage hunk")
				map("n", "<leader>gS", gs.stage_buffer, "Stage buffer")
				map("n", "<leader>gr", gs.reset_hunk, "Reset hunk")
				map("v", "<leader>gr", function()
					gs.reset_hunk({ vim.fn.line("."), vim.fn.line("v") })
				end, "Reset selected lines")
				map("n", "<leader>gR", gs.reset_buffer, "Reset buffer")

				-- Preview and blame
				map("n", "<leader>gp", gs.preview_hunk_inline, "Preview hunk inline")
				map("n", "<leader>gb", gs.blame_line, "Blame line")

				-- Hunk text object
				map({ "o", "x" }, "ih", gs.select_hunk, "Select hunk")

				-- Copy hunk to clipboard
				map("n", "<leader>yh", function()
					local hunks = gs.get_hunks(bufnr)
					if not hunks then
						return
					end
					local lnum = vim.fn.line(".")
					for _, h in ipairs(hunks) do
						local s = h.added.start
						local e = s + math.max(h.added.count, 1) - 1
						if lnum >= s and lnum <= e then
							local clean = vim.tbl_map(function(l)
								return l:sub(2)
							end, h.lines)
							vim.fn.setreg("+", table.concat(clean, "\n"))
							vim.notify("Copied hunk (" .. #h.lines .. " lines)")
							return
						end
					end
					vim.notify("No hunk at cursor", vim.log.levels.WARN)
				end, "Copy hunk")
				map("v", "<leader>yh", '"+y', "Copy selection")
			end,
		},
	},

	-- Smooth scrolling
	{
		"folke/snacks.nvim",
		lazy = false,
		opts = {
			indent = {
				enabled = true,
				animate = { enabled = false },
			},
			scope = { enabled = true },
			scroll = {
				animate = {
					duration = { step = 10, total = 100 },
					easing = "outQuad",
				},
				animate_repeat = {
					delay = 50,
					duration = { step = 5, total = 30 },
				},
			},
		},
	},

	-- Sticky scroll (shows containing function/class at top)
	{
		"nvim-treesitter/nvim-treesitter-context",
		dependencies = { "nvim-treesitter/nvim-treesitter" },
		opts = {
			max_lines = 3,
		},
	},

	-- Scroll past EOF
	{ "Aasim-A/scrollEOF.nvim", event = "CursorMoved", opts = {} },

	-- Scrollbar with git indicators
	{
		"petertriho/nvim-scrollbar",
		dependencies = { "lewis6991/gitsigns.nvim" },
		config = function()
			require("scrollbar").setup({
				handlers = { cursor = false },
			})
			require("scrollbar.handlers.gitsigns").setup()
		end,
	},

	-- Statusline
	{
		"nvim-lualine/lualine.nvim",
		dependencies = { "echasnovski/mini.icons" },
		opts = {
			options = {
				component_separators = "",
				section_separators = "",
			},
			sections = {
				lualine_a = { "mode" },
				lualine_b = {},
				lualine_c = { { "filename", path = 1 } },
				lualine_x = {},
				lualine_y = {},
				lualine_z = { "location" },
			},
			inactive_sections = {
				lualine_a = {},
				lualine_b = {},
				lualine_c = { { "filename", path = 1 } },
				lualine_x = { "location" },
				lualine_y = {},
				lualine_z = {},
			},
		},
	},

	-- Fuzzy finder
	{
		"nvim-telescope/telescope.nvim",
		dependencies = {
			"nvim-lua/plenary.nvim",
			{ "nvim-telescope/telescope-fzf-native.nvim", build = "make" },
			"nvim-telescope/telescope-frecency.nvim",
		},
		config = function()
			local telescope = require("telescope")
			local actions = require("telescope.actions")
			telescope.setup({
				defaults = {
					sorting_strategy = "ascending",
					layout_strategy = "vertical",
					layout_config = { preview_cutoff = 20, preview_height = 0.6, prompt_position = "top" },
					mappings = {
						i = {
							["<Esc>"] = actions.close,
							["<M-BS>"] = function(prompt_bufnr)
								local action_state = require("telescope.actions.state")
								local picker = action_state.get_current_picker(prompt_bufnr)
								local prompt = picker:_get_prompt()
								local cursor_col = vim.api.nvim_win_get_cursor(picker.prompt_win)[2]
									- #picker.prompt_prefix
								local before = prompt:sub(1, cursor_col)
								local trimmed = before:match("^(.-)%s*%S*$") or ""
								local after = prompt:sub(cursor_col + 1)
								picker:reset_prompt(trimmed .. after)
							end,
							["<C-u>"] = function(prompt_bufnr)
								local action_state = require("telescope.actions.state")
								local picker = action_state.get_current_picker(prompt_bufnr)
								picker:reset_prompt("")
							end,
						},
					},
				},
				pickers = {
					find_files = { hidden = true, file_ignore_patterns = { "%.git/" } },
				},
				extensions = {
					frecency = {
						show_filter_column = false,
					},
				},
			})
			telescope.load_extension("fzf")
			telescope.load_extension("frecency")
		end,
		keys = {
			{
				"<D-f>",
				function()
					require("telescope").extensions.frecency.frecency({ workspace = "CWD" })
				end,
				mode = { "n", "v", "i" },
				desc = "Find files",
			},
			{
				"<D-g>",
				function()
					require("telescope.builtin").live_grep()
				end,
				mode = { "n", "v", "i" },
				desc = "Ripgrep search",
			},
			{
				"<D-k>",
				function()
					require("telescope").extensions.frecency.frecency({ workspace = "CWD" })
				end,
				mode = { "n", "v", "i" },
				desc = "Find files",
			},
			{
				"<D-p>",
				function()
					require("telescope.builtin").commands()
				end,
				mode = { "n", "v", "i" },
				desc = "Command palette",
			},
			{
				"<leader>b",
				function()
					require("telescope.builtin").buffers()
				end,
				desc = "Find buffers",
			},
			{
				"<leader>f",
				function()
					require("telescope").extensions.frecency.frecency({ workspace = "CWD" })
				end,
				desc = "Find files",
			},
			{
				"<leader>m",
				function()
					require("telescope.builtin").keymaps()
				end,
				desc = "Keybindings",
			},
			{
				"<leader>p",
				function()
					require("telescope.builtin").commands()
				end,
				desc = "Command palette",
			},
			{
				"<leader>r",
				function()
					require("telescope.builtin").live_grep()
				end,
				desc = "Ripgrep search",
			},

			-- LSP pickers
			{
				"<D-S-o>",
				function()
					require("telescope.builtin").lsp_document_symbols()
				end,
				mode = { "n", "v", "i" },
				desc = "Document symbols",
			},
			{
				"<leader>ld",
				function()
					require("telescope.builtin").diagnostics()
				end,
				desc = "Diagnostics",
			},
			{
				"<leader>ls",
				function()
					require("telescope.builtin").lsp_document_symbols()
				end,
				desc = "Document symbols",
			},
			{
				"<leader>lS",
				function()
					require("telescope.builtin").lsp_workspace_symbols()
				end,
				desc = "Workspace symbols",
			},

			-- Git pickers
			{
				"<leader>gc",
				function()
					require("telescope.builtin").git_commits()
				end,
				desc = "Commits",
			},
			{
				"<leader>gC",
				function()
					require("telescope.builtin").git_bcommits()
				end,
				desc = "Buffer commits",
			},
			{
				"<leader>gB",
				function()
					require("telescope.builtin").git_branches()
				end,
				desc = "Branches",
			},
			{
				"<leader>gf",
				function()
					require("telescope.builtin").git_status()
				end,
				desc = "Changed files",
			},
		},
	},

	-- Window resize mode
	{
		"nvimtools/hydra.nvim",
		event = "VeryLazy",
		config = function()
			local Hydra = require("hydra")

			local function resize(dir)
				local amount = 2
				local cur = vim.fn.winnr()
				if dir == "h" or dir == "l" then
					local has_neighbor = vim.fn.winnr(dir) ~= cur
					if has_neighbor then
						vim.cmd(amount .. "wincmd >")
					else
						vim.cmd(amount .. "wincmd <")
					end
				else
					local has_neighbor = vim.fn.winnr(dir) ~= cur
					if has_neighbor then
						vim.cmd(amount .. "wincmd -")
					else
						vim.cmd(amount .. "wincmd +")
					end
				end
			end

			local hydra = Hydra({
				name = "Resize",
				mode = "n",
				body = "<C-w>r",
				heads = {
					{
						"h",
						function()
							resize("h")
						end,
						{ desc = "Left" },
					},
					{
						"l",
						function()
							resize("l")
						end,
						{ desc = "Right" },
					},
					{
						"j",
						function()
							resize("j")
						end,
						{ desc = "Down" },
					},
					{
						"k",
						function()
							resize("k")
						end,
						{ desc = "Up" },
					},
					{ "=", "<C-w>=", { desc = "Equalize" } },
					{ "<Esc>", nil, { exit = true, desc = "Exit" } },
				},
				config = {
					hint = { type = "statusline" },
					invoke_on_body = true,
					timeout = false,
				},
			})
			vim.keymap.set("n", "<C-r>", function()
				hydra:activate()
			end, { desc = "Resize mode" })
		end,
	},

	-- Multi-cursor (cmd+d select next, cmd+shift+d undo selection)
	{
		"mg979/vim-visual-multi",
		branch = "master",
		init = function()
			vim.g.VM_maps = {
				["Find Under"] = "<D-d>",
				["Find Subword Under"] = "<D-d>",
				["Remove Region"] = "<D-S-d>",
			}
		end,
	},

	-- GitHub Copilot
	{ "github/copilot.vim" },

	-- Completion
	{
		"saghen/blink.cmp",
		version = "1.*",
		opts = {
			keymap = {
				preset = "enter",
				["<Tab>"] = { "select_and_accept", "snippet_forward", "fallback" },
				["<S-Tab>"] = { "snippet_backward", "select_prev", "fallback" },
			},
			completion = {
				list = { selection = { preselect = true, auto_insert = false } },
			},
			sources = {
				default = { "lsp", "buffer" },
			},
			signature = { enabled = true },
		},
	},
	-- JSON schemas for intellisense
	{ "b0o/SchemaStore.nvim", lazy = true },

	{
		"stevearc/conform.nvim",
		event = "BufWritePre",
		opts = {
			formatters_by_ft = {
				go = { "gofumpt" },
				javascript = { "oxfmt" },
				javascriptreact = { "oxfmt" },
				typescript = { "oxfmt" },
				typescriptreact = { "oxfmt" },
				json = { "oxfmt" },
				yaml = { "oxfmt" },
				html = { "oxfmt" },
				css = { "oxfmt" },
				markdown = { "oxfmt" },
				rust = { "rustfmt" },
				sh = { "shfmt" },
				bash = { "shfmt" },
				fish = { "fish_indent" },
				lua = { "stylua" },
				nix = { "nixfmt" },
				terraform = { "terraform_fmt" },
				hcl = { "terraform_fmt" },
			},
			format_on_save = {
				timeout_ms = 3000,
				lsp_format = "fallback",
			},
		},
	},
})

-- ============================================================================
-- LSP
-- ============================================================================

vim.lsp.config("ts_ls", {
	cmd = { "typescript-language-server", "--stdio" },
	filetypes = { "javascript", "javascriptreact", "typescript", "typescriptreact" },
	root_markers = { "tsconfig.json", "jsconfig.json", "package.json", ".git" },
})

vim.lsp.enable("ts_ls")

vim.lsp.config("gopls", {
	cmd = { "gopls" },
	filetypes = { "go", "gomod", "gowork", "gotmpl" },
	root_markers = { "go.work", "go.mod", ".git" },
})

vim.lsp.enable("gopls")

vim.lsp.config("jsonls", {
	cmd = { "vscode-json-language-server", "--stdio" },
	filetypes = { "json", "jsonc" },
	root_markers = { ".git" },
	settings = {
		json = {
			schemas = require("schemastore").json.schemas(),
			validate = { enable = true },
		},
	},
})

vim.lsp.enable("jsonls")

vim.lsp.config("*", {
	capabilities = require("blink.cmp").get_lsp_capabilities(),
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
						require("telescope.builtin").lsp_references({ include_declaration = false })
					else
						require("telescope.builtin").lsp_definitions()
					end
				end)
			end)
		end, {
			buffer = args.buf,
			desc = "Go to definition / references",
		})
		vim.keymap.set("n", "gd", require("telescope.builtin").lsp_definitions, {
			buffer = args.buf,
			desc = "Go to definition (Telescope)",
		})
		vim.keymap.set("n", "grr", function()
			require("telescope.builtin").lsp_references({ include_declaration = false })
		end, {
			buffer = args.buf,
			desc = "Find references (Telescope)",
		})
		vim.keymap.set("n", "<C-Space>", vim.lsp.buf.hover, {
			buffer = args.buf,
			desc = "Hover documentation",
		})
		vim.keymap.set({ "n", "v", "i" }, "<D-.>", vim.lsp.buf.code_action, {
			buffer = args.buf,
			desc = "Code actions",
		})
		vim.keymap.set("n", "<M-r>", vim.lsp.buf.rename, {
			buffer = args.buf,
			desc = "Rename symbol",
		})
	end,
})
