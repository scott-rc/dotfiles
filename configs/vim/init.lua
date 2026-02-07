-- Source shared vimrc settings
vim.cmd('source ~/.vimrc')

-- Bootstrap lazy.nvim
local lazypath = vim.fn.stdpath('data') .. '/lazy/lazy.nvim'
if not vim.loop.fs_stat(lazypath) then
  vim.fn.system({
    'git', 'clone', '--filter=blob:none',
    'https://github.com/folke/lazy.nvim.git',
    '--branch=stable', lazypath,
  })
end
vim.opt.rtp:prepend(lazypath)

-- Plugins
require('lazy').setup({
  {
    'projekt0n/github-nvim-theme',
    lazy = false,
    priority = 1000,
    config = function()
      vim.cmd('colorscheme github_dark')
    end,
  },

  -- Existing plugins
  'dag/vim-fish',
  { 'junegunn/fzf', build = ':call fzf#install()' },
  'junegunn/fzf.vim',
  'tpope/vim-commentary',

  -- Completion (still needed - works with native LSP)
  {
    'hrsh7th/nvim-cmp',
    dependencies = {
      'hrsh7th/cmp-nvim-lsp',
      'hrsh7th/cmp-buffer',
    },
    config = function()
      local cmp = require('cmp')
      cmp.setup({
        mapping = cmp.mapping.preset.insert({
          ['<C-Space>'] = cmp.mapping.complete(),
          ['<CR>'] = cmp.mapping.confirm({ select = true }),
          ['<Tab>'] = cmp.mapping.select_next_item(),
          ['<S-Tab>'] = cmp.mapping.select_prev_item(),
        }),
        sources = {
          { name = 'nvim_lsp' },
          { name = 'buffer' },
        },
      })
    end,
  },
})

-- LSP Configuration (Neovim 0.11+ native API)
vim.lsp.config('ts_ls', {
  cmd = { 'typescript-language-server', '--stdio' },
  filetypes = { 'javascript', 'javascriptreact', 'typescript', 'typescriptreact' },
  root_markers = { 'tsconfig.json', 'jsconfig.json', 'package.json', '.git' },
})

-- Enable language servers (add more as needed)
vim.lsp.enable('ts_ls')

-- LSP keybindings on attach
vim.api.nvim_create_autocmd('LspAttach', {
  callback = function(args)
    local opts = { buffer = args.buf }
    -- gd for go-to-definition (not a default)
    vim.keymap.set('n', 'gd', vim.lsp.buf.definition, opts)
    -- [d and ]d for diagnostic navigation
    vim.keymap.set('n', '[d', vim.diagnostic.goto_prev, opts)
    vim.keymap.set('n', ']d', vim.diagnostic.goto_next, opts)
  end,
})

-- Advertise LSP capabilities for completion
vim.lsp.config('*', {
  capabilities = require('cmp_nvim_lsp').default_capabilities(),
})

vim.g.fzf_vim = { preview_window = {'up,60%', 'ctrl-/'} }

-- FZF mappings
vim.keymap.set('n', '<leader>f', ':Files<CR>')
vim.keymap.set('n', '<leader>b', ':Buffers<CR>')
vim.keymap.set('n', '<leader>r', ':Rg<CR>')

-- Commentary mappings
vim.keymap.set('n', '<leader>/', '<Plug>CommentaryLine')
vim.keymap.set('v', '<leader>/', '<Plug>Commentary')
