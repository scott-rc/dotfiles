return {
	{
		"saghen/blink.cmp",
		version = "1.*",
		opts = {
			appearance = {
				nerd_font_variant = "mono",
			},
			keymap = {
				preset = "enter",
				["<Tab>"] = { "select_and_accept", "snippet_forward", "fallback" },
				["<S-Tab>"] = { "snippet_backward", "select_prev", "fallback" },
			},
			completion = {
				documentation = {
					auto_show = true,
					window = {
						border = "rounded",
					},
				},
				ghost_text = {
					enabled = true,
				},
				list = {
					selection = {
						preselect = true,
						auto_insert = false,
					},
				},
				menu = {
					border = "rounded",
					scrollbar = false,
					draw = {
						padding = 1,
						treesitter = { "lsp" },
						cursorline_priority = 20001,
						columns = {
							{ "kind_icon" },
							{ "label", "label_description", gap = 1 },
						},
					},
				},
			},
			cmdline = {
				keymap = {
					preset = "cmdline",
					["<Down>"] = { "select_next", "fallback" },
					["<Up>"] = { "select_prev", "fallback" },
				},
			},
			sources = {
				default = { "lsp", "buffer" },
			},
			signature = {
				enabled = true,
				window = {
					border = "rounded",
				},
			},
		},
	},

	-- JSON schemas for intellisense
	{ "b0o/SchemaStore.nvim", lazy = true },
}
