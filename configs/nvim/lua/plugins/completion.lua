return {
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
}
