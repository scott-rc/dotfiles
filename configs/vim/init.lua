-- ============================================================================
-- Settings
-- ============================================================================

vim.g.mapleader = ' '
vim.g.vim_json_conceal = 0

vim.opt.hidden = true
vim.opt.encoding = 'utf-8'
vim.opt.ignorecase = true
vim.opt.smartcase = true
vim.opt.gdefault = true
vim.opt.splitbelow = true
vim.opt.splitright = true
vim.opt.showmatch = true
vim.opt.number = true
vim.opt.formatoptions:append('o')
vim.opt.expandtab = true
vim.opt.tabstop = 4
vim.opt.shiftwidth = 4
vim.opt.autoindent = true
vim.opt.wrap = false
vim.opt.joinspaces = false
vim.opt.mouse = 'n'
vim.opt.updatetime = 750
vim.opt.undofile = true
vim.opt.undodir = vim.fn.expand('~/.vim/undo')
vim.opt.history = 500
vim.opt.wildmenu = true
vim.opt.backspace = { 'eol', 'start', 'indent' }
vim.opt.autoread = true
vim.opt.ruler = true
vim.opt.incsearch = true
vim.opt.lazyredraw = true
vim.opt.whichwrap:append('<,>,h,l')
vim.opt.visualbell = false
vim.opt.errorbells = false
vim.opt.backup = false
vim.opt.writebackup = false
vim.opt.swapfile = false
vim.opt.scrolloff = 8

-- ============================================================================
-- Keybindings
-- ============================================================================

-- Escape
vim.keymap.set({ 'n', 'v', 'i' }, 'fd', '<Esc>')

-- Navigation
vim.keymap.set('n', ';', ':')
vim.keymap.set({ 'n', 'v' }, '<S-h>', '^')
vim.keymap.set({ 'n', 'v' }, '<S-l>', '$')

-- Leader
vim.keymap.set('n', '<leader>q', '<cmd>q<CR>', { desc = 'Quit' })
vim.keymap.set('n', '<leader>w', '<cmd>q<CR>', { desc = 'Quit' })
vim.keymap.set('n', '<leader>s', '<cmd>w<CR>', { desc = 'Save' })
vim.keymap.set('n', '<leader>t', '<cmd>tabnew<CR>', { desc = 'New tab' })
vim.keymap.set('n', '<leader>[', '<cmd>tabp<CR>', { desc = 'Previous tab' })
vim.keymap.set('n', '<leader>]', '<cmd>tabn<CR>', { desc = 'Next tab' })
vim.keymap.set('n', '<leader>%', '<cmd>source %<CR>', { desc = 'Source file' })
vim.keymap.set('v', '<leader>y', '"+y', { desc = 'Copy to clipboard' })

-- Wildmenu navigation
vim.opt.wildcharm = vim.fn.char2nr(vim.api.nvim_replace_termcodes('<C-z>', true, true, true))
vim.keymap.set('c', '<up>', function() return vim.fn.wildmenumode() == 1 and '<left>' or '<up>' end, { expr = true })
vim.keymap.set('c', '<down>', function() return vim.fn.wildmenumode() == 1 and '<right>' or '<down>' end, { expr = true })
vim.keymap.set('c', '<left>', function() return vim.fn.wildmenumode() == 1 and '<up>' or '<left>' end, { expr = true })
vim.keymap.set('c', '<right>', function() return vim.fn.wildmenumode() == 1 and ' <bs><C-z>' or '<right>' end, { expr = true })

-- ============================================================================
-- Autocommands
-- ============================================================================

vim.api.nvim_create_autocmd({ 'FocusGained', 'BufEnter' }, {
  command = 'checktime',
})

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

  -- Keybinding popup
  {
    'folke/which-key.nvim',
    event = 'VeryLazy',
    opts = {
      preset = 'modern',
    },
  },

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
