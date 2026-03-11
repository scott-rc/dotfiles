return {
	-- Icons (file + folder-name-specific, e.g. src, test, node_modules)
	{
		"echasnovski/mini.icons",
		opts = {},
		config = function(_, opts)
			require("mini.icons").setup(opts)
			MiniIcons.mock_nvim_web_devicons()
		end,
	},

	-- Smooth scrolling + indent guides
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
			explorer = {
				replace_netrw = true,
			},
			picker = {
				enabled = true,
				matcher = {
					frecency = true,
					cwd_bonus = true,
					smartcase = true,
				},
				layout = {
					preset = "vertical",
					cycle = true,
				},
				layouts = {
					vertical = {
						layout = {
							backdrop = false,
							width = 0.5,
							min_width = 80,
							height = 0.8,
							min_height = 30,
							box = "vertical",
							border = true,
							title = "{title} {live} {flags}",
							title_pos = "center",
							{ win = "input", height = 1, border = "bottom" },
							{ win = "list", border = "none" },
							{ win = "preview", title = "{preview}", height = 0.6, border = "top" },
						},
					},
				},
				sources = {
					files = { hidden = true },
					explorer = {
						follow_file = true,
						auto_close = false,
						jump = { close = false },
						layout = { layout = { position = "right" } },
						win = {
							list = {
								keys = {
									["<CR>"] = { "confirm", function(picker) picker:focus("list") end },
									["l"] = { "confirm", function(picker) picker:focus("list") end },
								},
							},
						},
					},
				},
				win = {
					input = {
						keys = {
							["<Esc>"] = { "close", mode = { "n", "i" } },
							["<A-BS>"] = { "<c-w>", mode = { "i" }, expr = true, desc = "delete word" },
							["<C-u>"] = { "<c-u>", mode = { "i" }, expr = true, desc = "clear prompt" },
						},
					},
				},
			},
		},
		keys = {
			-- File / grep
			{ "<D-f>", function() Snacks.picker.smart() end, mode = { "n", "v", "i" }, desc = "Smart find files" },
			{ "<D-g>", function() Snacks.picker.grep() end, mode = { "n", "v", "i" }, desc = "Live grep" },
			{ "<D-k>", function() Snacks.picker.smart() end, mode = { "n", "v", "i" }, desc = "Smart find files" },
			{ "<D-p>", function() Snacks.picker.commands() end, mode = { "n", "v", "i" }, desc = "Commands" },
			{ "<leader>b", function() Snacks.picker.buffers() end, desc = "Buffers" },
			{ "<leader>f", function() Snacks.picker.smart() end, desc = "Smart find files" },
			{ "<leader>m", function() Snacks.picker.keymaps() end, desc = "Keymaps" },
			{ "<leader>p", function() Snacks.picker.commands() end, desc = "Commands" },
			{ "<leader>r", function() Snacks.picker.grep() end, desc = "Live grep" },
			-- LSP
			{ "<D-S-o>", function() Snacks.picker.lsp_symbols() end, mode = { "n", "v", "i" }, desc = "LSP document symbols" },
			{ "<leader>ld", function() Snacks.picker.diagnostics() end, desc = "Diagnostics" },
			{ "<leader>ls", function() Snacks.picker.lsp_symbols() end, desc = "LSP document symbols" },
			{ "<leader>lS", function() Snacks.picker.lsp_workspace_symbols() end, desc = "LSP workspace symbols" },
			-- Git
			{ "<leader>gc", function() Snacks.picker.git_log() end, desc = "Git commits" },
			{ "<leader>gC", function() Snacks.picker.git_log_file() end, desc = "Git buffer commits" },
			{ "<leader>gB", function() Snacks.picker.git_branches() end, desc = "Git branches" },
			{ "<leader>gf", function() Snacks.picker.git_status() end, desc = "Git status" },
			-- Explorer
			{ "<leader>e", function() Snacks.explorer() end, desc = "Explorer" },
			{ "<C-e>", function() Snacks.explorer() end, desc = "Explorer" },
			{ "<D-e>", function() Snacks.explorer() end, mode = { "n", "v", "i" }, desc = "Explorer" },
		},
	},

	-- Scroll past EOF
	{ "Aasim-A/scrollEOF.nvim", event = "CursorMoved", opts = {} },

	-- Scrollbar with git indicators
	{
		"lewis6991/satellite.nvim",
		dependencies = { "lewis6991/gitsigns.nvim" },
		opts = {
			handlers = {
				cursor = { enable = false },
				gitsigns = { enable = true },
			},
		},
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
}
