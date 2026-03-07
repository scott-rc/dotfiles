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
}
