vim.g.vim_json_conceal = 0
vim.o.timeoutlen = 300

-- Search
vim.o.ignorecase = true
vim.o.smartcase = true
vim.o.gdefault = true

-- Splits
vim.o.splitbelow = true
vim.o.splitright = true
vim.o.splitkeep = "screen"

-- Display
vim.o.title = true
vim.o.titlestring = " %f%( %m%)"
vim.o.cmdheight = 0
vim.o.showmode = false
vim.o.laststatus = 3
vim.o.signcolumn = "yes"
vim.o.showmatch = true
vim.o.number = true
vim.o.cursorline = true
vim.o.wrap = false
vim.o.scrolloff = 999
vim.opt.listchars = { tab = "→ ", trail = "·", nbsp = "+", extends = ">", precedes = "<" }

-- Indentation
vim.o.expandtab = true
vim.o.tabstop = 4
vim.o.shiftwidth = 4
vim.o.smartindent = true
vim.opt.formatoptions:append("o")

-- Folding (treesitter-based, all open by default)
vim.o.foldmethod = "expr"
vim.o.foldexpr = "v:lua.vim.treesitter.foldexpr()"
vim.o.foldlevelstart = 99
vim.o.foldtext = ""
vim.o.fillchars = "fold: ,foldopen:▼,foldclose:▶,foldsep:│"

-- Behavior
vim.o.autoread = true
vim.o.mouse = "n"
vim.o.updatetime = 250
vim.opt.whichwrap:append("<,>,h,l")

-- Statusline
vim.o.statusline = table.concat({
	"%{%v:lua.require'config.statusline'.mode()%}",
	" %<%f %m%r",
	"%=",
	"%{v:lua.vim.diagnostic.status()} ",
	"%{v:lua.vim.lsp.status()} ",
	"%l:%c ",
})

-- Native completion (alternative to blink.cmp — uncomment to try)
-- Disable blink.cmp first, then enable these:
-- vim.o.autocomplete = true
-- vim.o.pumborder = "rounded"
-- vim.o.pummaxwidth = 40
-- vim.o.completeopt = "menuone,popup,fuzzy,noselect,nearest"
-- Also flip copilot.lua: suggestion = { enabled = true, auto_trigger = true }
-- Also change lsp.lua: capabilities = vim.lsp.protocol.make_client_capabilities()

-- Undotree
vim.cmd.packadd("nvim.undotree")

-- Persistence
vim.o.undofile = true
vim.o.writebackup = false
vim.o.swapfile = false
