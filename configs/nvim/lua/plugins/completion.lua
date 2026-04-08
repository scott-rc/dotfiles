require("blink.cmp").setup({
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
		default = { "lsp", "copilot", "buffer" },
		providers = {
			copilot = {
				name = "copilot",
				module = "blink-copilot",
				score_offset = 100,
				async = true,
			},
		},
	},
	signature = {
		enabled = true,
		window = {
			border = "rounded",
		},
	},
})

-- SchemaStore loaded on demand by lsp/jsonls.lua
