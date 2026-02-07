-- Source shared vimrc settings
vim.cmd('source ~/.vimrc')

-- ============================================================================
-- Bootstrap lazy.nvim
-- ============================================================================
local lazypath = vim.fn.stdpath('data') .. '/lazy/lazy.nvim'
if not vim.uv.fs_stat(lazypath) then
  vim.fn.system({
    'git', 'clone', '--filter=blob:none',
    'https://github.com/folke/lazy.nvim.git',
    '--branch=stable', lazypath,
  })
end
vim.opt.rtp:prepend(lazypath)

-- ============================================================================
-- Plugins
-- ============================================================================
require('lazy').setup({
  -- Theme
  {
    'projekt0n/github-nvim-theme',
    lazy = false,
    priority = 1000,
    config = function()
      vim.cmd('colorscheme github_dark')
    end,
  },

  -- Syntax
  'dag/vim-fish',

  -- Fuzzy finder
  { 'junegunn/fzf', build = ':call fzf#install()' },
  {
    'junegunn/fzf.vim',
    config = function()
      vim.g.fzf_vim = { preview_window = { 'up,60%', 'ctrl-/' } }
    end,
    keys = {
      { '<leader>f', '<cmd>Files<CR>', desc = 'Find files' },
      { '<leader>b', '<cmd>Buffers<CR>', desc = 'Find buffers' },
      { '<leader>r', '<cmd>Rg<CR>', desc = 'Ripgrep search' },
    },
  },

  -- Commenting
  {
    'tpope/vim-commentary',
    keys = {
      { '<leader>/', '<Plug>CommentaryLine', desc = 'Toggle comment' },
      { '<leader>/', '<Plug>Commentary', mode = 'v', desc = 'Toggle comment' },
    },
  },

  -- Completion
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
      vim.lsp.config('*', {
        capabilities = require('cmp_nvim_lsp').default_capabilities(),
      })
    end,
  },
})

-- ============================================================================
-- LSP
-- ============================================================================
vim.lsp.config('ts_ls', {
  cmd = { 'typescript-language-server', '--stdio' },
  filetypes = { 'javascript', 'javascriptreact', 'typescript', 'typescriptreact' },
  root_markers = { 'tsconfig.json', 'jsconfig.json', 'package.json', '.git' },
})

vim.lsp.enable('ts_ls')

vim.api.nvim_create_autocmd('LspAttach', {
  callback = function(args)
    vim.keymap.set('n', 'gd', vim.lsp.buf.definition, {
      buffer = args.buf,
      desc = 'Go to definition',
    })
  end,
})
