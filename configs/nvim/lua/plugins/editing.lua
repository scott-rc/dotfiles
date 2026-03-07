return {
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

	-- Auto-detect indentation (tabstop, shiftwidth, expandtab)
	{ "tpope/vim-sleuth" },

	-- GitHub Copilot
	{ "github/copilot.vim" },
}
