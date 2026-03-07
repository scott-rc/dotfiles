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

local neotree_augroup = vim.api.nvim_create_augroup("neotree_config", { clear = true })

local function neotree_state_file()
	local cwd = vim.uv.cwd()
	local hash = vim.fn.sha256(cwd):sub(1, 12)
	return vim.fn.stdpath("data") .. "/neotree-open-" .. hash
end

-- When opened with a directory arg (e.g. `nvim .`), replace the directory
-- buffer with an empty buffer and open neo-tree as a proper sidebar
vim.api.nvim_create_autocmd("VimEnter", {
	group = neotree_augroup,
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

-- Lock neo-tree sidebar width so equalalways doesn't resize it during focus changes
vim.api.nvim_create_autocmd("FileType", {
	group = neotree_augroup,
	pattern = "neo-tree",
	callback = function()
		vim.wo.winfixwidth = true
	end,
})

-- Persist neo-tree open/closed state per working directory
vim.api.nvim_create_autocmd("VimLeavePre", {
	group = neotree_augroup,
	callback = function()
		local state_file = neotree_state_file()
		local ok, manager = pcall(require, "neo-tree.sources.manager")
		if ok then
			local state = manager.get_state("filesystem")
			if state.winid and vim.api.nvim_win_is_valid(state.winid) then
				vim.fn.writefile({}, state_file)
				return
			end
		end
		vim.fn.delete(state_file)
	end,
})

vim.api.nvim_create_autocmd("VimEnter", {
	group = neotree_augroup,
	callback = function()
		-- Only restore if opening a file (not a directory, which already opens neo-tree)
		if vim.fn.argc() == 1 and vim.fn.isdirectory(vim.fn.argv(0)) == 1 then
			return
		end
		if vim.uv.fs_stat(neotree_state_file()) then
			vim.schedule(function()
				require("neo-tree")
				vim.cmd("Neotree source=filesystem")
			end)
		end
	end,
})

return {
	"nvim-neo-tree/neo-tree.nvim",
	branch = "v3.x",
	dependencies = {
		"nvim-lua/plenary.nvim",
		"MunifTanjim/nui.nvim",
		"echasnovski/mini.icons",
	},
	config = function()
		-- Monkey-patch: neo-tree crashes on deleted files whose parent directory
		-- was also deleted (fs_lstat returns nil -> type "unknown" -> no children
		-- table -> table.insert crashes). Force non-existent extensionless paths
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
}
