vim.lsp.config("*", {
	capabilities = require("blink.cmp").get_lsp_capabilities(),
})

vim.lsp.enable({ "ts_ls", "gopls", "jsonls", "cue", "jinja_lsp" })
