local grep_ref = { "" }
local files_ref = { "" }

local function save_prompt_mappings(ref)
	local actions = require("telescope.actions")
	local action_state = require("telescope.actions.state")
	return function(_, map)
		local function save(bufnr)
			ref[1] = action_state.get_current_picker(bufnr):_get_prompt()
		end
		map("i", "<Esc>", function(bufnr)
			save(bufnr)
			actions.close(bufnr)
		end)
		map("i", "<CR>", function(bufnr)
			save(bufnr)
			actions.select_default(bufnr)
		end)
		return true
	end
end

local function live_grep_with_last()
	require("telescope.builtin").live_grep({
		default_text = grep_ref[1],
		attach_mappings = save_prompt_mappings(grep_ref),
	})
end

local function find_files_with_last()
	require("telescope").extensions.frecency.frecency({
		workspace = "CWD",
		default_text = files_ref[1],
		attach_mappings = save_prompt_mappings(files_ref),
	})
end

return {
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
					db_safe_mode = false,
				},
			},
		})
		telescope.load_extension("fzf")
		telescope.load_extension("frecency")
	end,
	keys = {
		{ "<D-f>", find_files_with_last, mode = { "n", "v", "i" }, desc = "Find files (last search)" },
		{ "<D-g>", live_grep_with_last, mode = { "n", "v", "i" }, desc = "Ripgrep search (last search)" },
		{ "<D-k>", find_files_with_last, mode = { "n", "v", "i" }, desc = "Find files (last search)" },
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
}
