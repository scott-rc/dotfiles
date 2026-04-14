vim.lsp.config("*", {
	capabilities = require("blink.cmp").get_lsp_capabilities(),
	codelens = { enabled = true },
})

vim.diagnostic.config({
	virtual_lines = { severity = vim.diagnostic.severity.ERROR },
})

vim.lsp.enable({ "ts_ls", "gopls", "jsonls", "cue", "jinja_lsp", "rust_analyzer" })
