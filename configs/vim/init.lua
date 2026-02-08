-- ============================================================================
-- Settings
-- ============================================================================

vim.g.mapleader = ' '
vim.g.vim_json_conceal = 0

-- Search
vim.o.ignorecase = true
vim.o.smartcase = true
vim.o.gdefault = true

-- Splits
vim.o.splitbelow = true
vim.o.splitright = true

-- Display
vim.o.showmatch = true
vim.o.number = true
vim.o.wrap = false
vim.o.lazyredraw = true
vim.o.scrolloff = 8

-- Indentation
vim.o.expandtab = true
vim.o.tabstop = 4
vim.o.shiftwidth = 4
vim.opt.formatoptions:append('o')

-- Behavior
vim.o.mouse = 'n'
vim.o.updatetime = 750
vim.opt.whichwrap:append('<,>,h,l')

-- Persistence
vim.o.undofile = true
vim.o.writebackup = false
vim.o.swapfile = false

-- ============================================================================
-- Keybindings
-- ============================================================================

-- Escape
vim.keymap.set({ 'n', 'v', 'i' }, 'fd', '<Esc>', { desc = 'Escape' })

-- Navigation
vim.keymap.set('n', ';', ':', { desc = 'Command mode' })
vim.keymap.set({ 'n', 'v' }, '<S-h>', '^', { desc = 'Start of line' })
vim.keymap.set({ 'n', 'v' }, '<S-l>', '$', { desc = 'End of line' })

-- Leader
vim.keymap.set('n', '<leader>w', '<cmd>q<CR>', { desc = 'Close' })
vim.keymap.set('n', '<leader>q', '<cmd>qa<CR>', { desc = 'Quit all' })
vim.keymap.set('n', '<leader>s', '<cmd>w<CR>', { desc = 'Save' })
vim.keymap.set('n', '<leader>t', '<cmd>tabnew<CR>', { desc = 'New tab' })
vim.keymap.set('n', '<leader>[', '<cmd>tabp<CR>', { desc = 'Previous tab' })
vim.keymap.set('n', '<leader>]', '<cmd>tabn<CR>', { desc = 'Next tab' })
vim.keymap.set('n', '<leader>%', '<cmd>source %<CR>', { desc = 'Source file' })
vim.keymap.set('v', '<leader>y', '"+y', { desc = 'Copy to clipboard' })

-- Wildmenu navigation
vim.o.wildcharm = vim.fn.char2nr(vim.api.nvim_replace_termcodes('<C-z>', true, true, true))
vim.keymap.set('c', '<up>', function() return vim.fn.wildmenumode() == 1 and '<left>' or '<up>' end, { expr = true, desc = 'Wildmenu: previous match' })
vim.keymap.set('c', '<down>', function() return vim.fn.wildmenumode() == 1 and '<right>' or '<down>' end, { expr = true, desc = 'Wildmenu: next match' })
vim.keymap.set('c', '<left>', function() return vim.fn.wildmenumode() == 1 and '<up>' or '<left>' end, { expr = true, desc = 'Wildmenu: parent dir' })
vim.keymap.set('c', '<right>', function() return vim.fn.wildmenumode() == 1 and ' <bs><C-z>' or '<right>' end, { expr = true, desc = 'Wildmenu: enter dir' })

-- ============================================================================
-- Autocommands
-- ============================================================================

local augroup = vim.api.nvim_create_augroup('user_config', { clear = true })

vim.api.nvim_create_autocmd('VimEnter', {
  group = augroup,
  callback = function()
    if vim.fn.argc() == 0 and vim.bo.buftype == '' then
      local root = vim.fn.systemlist('git rev-parse --show-toplevel')[1]
      if vim.v.shell_error == 0 and root and root ~= '' then
        require('neo-tree.command').execute({ action = 'focus', dir = root })
      else
        require('neo-tree.command').execute({ action = 'focus' })
      end
      vim.cmd('only')
    end
  end,
})

vim.api.nvim_create_autocmd({ 'FocusGained', 'BufEnter' }, {
  group = augroup,
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
      vim.cmd.colorscheme('github_dark')
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

  -- File explorer
  {
    'nvim-neo-tree/neo-tree.nvim',
    branch = 'v3.x',
    dependencies = {
      'nvim-lua/plenary.nvim',
      'MunifTanjim/nui.nvim',
      'nvim-tree/nvim-web-devicons',
    },
    opts = {
      filesystem = {
        follow_current_file = { enabled = true },
        use_libuv_file_watcher = true,
      },
    },
    keys = {
      { '<leader>e', '<cmd>Neotree toggle<CR>', desc = 'File explorer' },
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

-- Only gd is mapped here; grn, gra, grr, K are Neovim 0.11+ built-in LSP defaults
vim.api.nvim_create_autocmd('LspAttach', {
  group = augroup,
  callback = function(args)
    vim.keymap.set('n', 'gd', vim.lsp.buf.definition, {
      buffer = args.buf,
      desc = 'Go to definition',
    })
  end,
})
