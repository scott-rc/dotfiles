-- ============================================================================
-- Settings
-- ============================================================================

vim.g.mapleader = ' '
vim.g.vim_json_conceal = 0
vim.o.timeoutlen = 300

-- Search
vim.o.ignorecase = true
vim.o.smartcase = true
vim.o.gdefault = true

-- Splits
vim.o.splitbelow = true
vim.o.splitright = true

-- Display
vim.o.showmode = false
vim.o.showmatch = true
vim.o.number = true
vim.o.wrap = false
vim.o.lazyredraw = true
vim.o.scrolloff = 8
vim.opt.listchars = { tab = '→ ', trail = '·', nbsp = '+', extends = '>', precedes = '<' }

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

-- Option+Delete word deletion (Ghostty sends Alt/ESC prefix for Option key)
vim.keymap.set('i', '<M-BS>', '<C-w>', { desc = 'Delete word backward' })

-- Navigate by display lines when no count is given (for wrapped lines)
vim.keymap.set({ 'n', 'v' }, 'j', function() return vim.v.count == 0 and 'gj' or 'j' end, { expr = true, desc = 'Down (wrap-aware)' })
vim.keymap.set({ 'n', 'v' }, 'k', function() return vim.v.count == 0 and 'gk' or 'k' end, { expr = true, desc = 'Up (wrap-aware)' })

-- Navigation
vim.keymap.set('n', ';', ':', { desc = 'Command mode' })
vim.keymap.set({ 'n', 'v' }, '<S-h>', '^', { desc = 'Start of line' })
vim.keymap.set({ 'n', 'v' }, '<S-l>', '$', { desc = 'End of line' })

-- Window navigation
vim.keymap.set('n', '<C-h>', '<C-w>h', { desc = 'Focus left window' })
vim.keymap.set('n', '<C-j>', '<C-w>j', { desc = 'Focus below window' })
vim.keymap.set('n', '<C-k>', '<C-w>k', { desc = 'Focus above window' })
vim.keymap.set('n', '<C-l>', '<C-w>l', { desc = 'Focus right window' })

-- Leader
vim.keymap.set('n', '<leader>w', '<cmd>q<CR>', { desc = 'Close' })
vim.keymap.set('n', '<leader>q', '<cmd>qa<CR>', { desc = 'Quit all' })
vim.keymap.set('n', '<leader>s', '<cmd>w<CR>', { desc = 'Save' })
vim.keymap.set('n', '<leader>t', '<cmd>tabnew<CR>', { desc = 'New tab' })
vim.keymap.set('n', '<leader>[', '<cmd>tabp<CR>', { desc = 'Previous tab' })
vim.keymap.set('n', '<leader>]', '<cmd>tabn<CR>', { desc = 'Next tab' })
vim.keymap.set('n', '<leader>%', '<cmd>source %<CR>', { desc = 'Source file' })
vim.keymap.set('v', '<D-c>', '"+y', { desc = 'Copy to clipboard' })
vim.keymap.set('n', '<D-q>', '<cmd>qa<CR>', { desc = 'Quit' })
vim.keymap.set({ 'n', 'i' }, '<D-s>', '<cmd>w<CR>', { desc = 'Save' })
vim.keymap.set('n', '<D-w>', '<cmd>bdelete<CR>', { desc = 'Close buffer' })
vim.keymap.set('n', '<leader>yp', function() vim.fn.setreg('+', vim.fn.fnamemodify(vim.fn.expand('%'), ':.')) end, { desc = 'Copy relative path' })
vim.keymap.set('n', '<leader>yP', function() vim.fn.setreg('+', vim.fn.expand('%:p')) end, { desc = 'Copy absolute path' })
vim.keymap.set('n', '<leader>yl', function() vim.fn.setreg('+', vim.fn.fnamemodify(vim.fn.expand('%'), ':.') .. ':' .. vim.fn.line('.')) end, { desc = 'Copy relative path:line' })
vim.keymap.set('n', '<leader>yL', function() vim.fn.setreg('+', vim.fn.expand('%:p') .. ':' .. vim.fn.line('.')) end, { desc = 'Copy absolute path:line' })
vim.keymap.set('v', '<leader>yp', function() vim.fn.setreg('+', vim.fn.fnamemodify(vim.fn.expand('%'), ':.')) end, { desc = 'Copy relative path' })
vim.keymap.set('v', '<leader>yP', function() vim.fn.setreg('+', vim.fn.expand('%:p')) end, { desc = 'Copy absolute path' })
vim.keymap.set('v', '<leader>yl', function() vim.fn.setreg('+', vim.fn.fnamemodify(vim.fn.expand('%'), ':.') .. ':' .. vim.fn.line("'<") .. '-' .. vim.fn.line("'>")) end, { desc = 'Copy relative path:lines' })
vim.keymap.set('v', '<leader>yL', function() vim.fn.setreg('+', vim.fn.expand('%:p') .. ':' .. vim.fn.line("'<") .. '-' .. vim.fn.line("'>")) end, { desc = 'Copy absolute path:lines' })

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
        require('neo-tree.command').execute({ action = 'show', dir = root })
      else
        require('neo-tree.command').execute({ action = 'show' })
      end
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
      local function is_dark_mode()
        local result = vim.fn.system('defaults read -g AppleInterfaceStyle 2>/dev/null')
        return result:match('Dark') ~= nil
      end

      local function apply_theme()
        if is_dark_mode() then
          vim.cmd.colorscheme('github_dark_default')
        else
          vim.cmd.colorscheme('github_dark_dimmed')
        end
      end

      apply_theme()

      -- Re-check on focus so theme follows system appearance changes
      vim.api.nvim_create_autocmd('FocusGained', {
        callback = apply_theme,
      })
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
      spec = {
        { '<leader>y', group = 'Yank path' },

        { '<leader>o', group = 'Options' },
        { '<leader>ow', function()
          local wo = vim.wo
          wo.wrap = not wo.wrap
          wo.linebreak = wo.wrap
          wo.breakindent = wo.wrap
          wo.colorcolumn = wo.wrap and '100' or ''
        end, desc = 'Toggle wrap' },
        { '<leader>on', '<cmd>set number!<cr>',          desc = 'Toggle line numbers' },
        { '<leader>or', '<cmd>set relativenumber!<cr>',  desc = 'Toggle relative numbers' },
        { '<leader>oh', '<cmd>nohlsearch<cr>',           desc = 'Clear search highlight' },
        { '<leader>os', '<cmd>set spell!<cr>',           desc = 'Toggle spell check' },
        { '<leader>ol', '<cmd>set list!<cr>',            desc = 'Toggle invisible chars' },
        { '<leader>oi', '<cmd>set ignorecase!<cr>',      desc = 'Toggle ignore case' },
      },
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
      window = {
        position = 'right',
        mappings = {
          ['<space>'] = 'none',
          ['<cr>'] = function(state)
            local node = state.tree:get_node()
            require('neo-tree.sources.' .. state.name .. '.commands').open(state)
            if node.type ~= 'directory' then
              vim.cmd('Neotree reveal source=' .. state.name)
            end
          end,
        },
      },
      filesystem = {
        follow_current_file = { enabled = true },
        use_libuv_file_watcher = true,
      },
      git_status = {
        window = {
          mappings = {
            ['<cr>'] = function(state)
              local node = state.tree:get_node()
              require('neo-tree.sources.' .. state.name .. '.commands').open(state)
              if node.type ~= 'directory' then
                vim.b.diff_base = state.git_base or 'HEAD'
                vim.cmd('Neotree reveal source=' .. state.name)
              end
            end,
          },
        },
      },
    },
    keys = {
      { '<leader>e', '<cmd>Neotree toggle<CR>', desc = 'File explorer' },
      { '<C-e>', function()
          if vim.bo.filetype == 'neo-tree' and vim.b.neo_tree_source == 'filesystem' then
            vim.cmd('Neotree close')
          else
            vim.cmd('Neotree focus source=filesystem')
          end
        end, desc = 'Focus/toggle file explorer' },
      { '<D-e>', function()
          if vim.bo.filetype == 'neo-tree' and vim.b.neo_tree_source == 'filesystem' then
            vim.cmd('Neotree close')
          else
            vim.cmd('Neotree focus source=filesystem')
          end
        end, desc = 'Focus/toggle file explorer' },
      {
        '<leader>g',
        function()
          local handle = io.popen('git rev-parse --abbrev-ref origin/HEAD 2>/dev/null')
          local base = 'main'
          if handle then
            local result = handle:read('*a'):gsub('%s+', '')
            handle:close()
            local branch = result:match('origin/(.*)')
            if branch and branch ~= '' then
              base = branch
            end
          end
          vim.cmd('Neotree git_status git_base=' .. base)
        end,
        desc = 'Changed files vs base branch',
      },
      { '<D-g>', function()
          if vim.bo.filetype == 'neo-tree' and vim.b.neo_tree_source == 'git_status' then
            vim.cmd('Neotree close')
          else
            local handle = io.popen('git rev-parse --abbrev-ref origin/HEAD 2>/dev/null')
            local base = 'main'
            if handle then
              local result = handle:read('*a'):gsub('%s+', '')
              handle:close()
              local branch = result:match('origin/(.*)')
              if branch and branch ~= '' then base = branch end
            end
            vim.cmd('Neotree focus source=git_status git_base=' .. base)
          end
        end, desc = 'Focus/toggle git changes' },
      { '<C-g>', function()
          if vim.bo.filetype == 'neo-tree' and vim.b.neo_tree_source == 'git_status' then
            vim.cmd('Neotree close')
          else
            local handle = io.popen('git rev-parse --abbrev-ref origin/HEAD 2>/dev/null')
            local base = 'main'
            if handle then
              local result = handle:read('*a'):gsub('%s+', '')
              handle:close()
              local branch = result:match('origin/(.*)')
              if branch and branch ~= '' then base = branch end
            end
            vim.cmd('Neotree focus source=git_status git_base=' .. base)
          end
        end, desc = 'Focus/toggle git changes' },
    },
  },

  -- Git change indicators
  {
    'lewis6991/gitsigns.nvim',
    opts = {
      -- show_deleted = true,
      on_attach = function(bufnr)
        local base = vim.b[bufnr].diff_base
        if base then
          require('gitsigns').change_base(base)
        end
      end,
    },
  },

  -- Scrollbar with git indicators
  {
    'petertriho/nvim-scrollbar',
    dependencies = { 'lewis6991/gitsigns.nvim' },
    config = function()
      require('scrollbar').setup()
      require('scrollbar.handlers.gitsigns').setup()
    end,
  },

  -- Statusline
  {
    'nvim-lualine/lualine.nvim',
    dependencies = { 'nvim-tree/nvim-web-devicons' },
    opts = {},
  },

  -- Fuzzy finder
  {
    'nvim-telescope/telescope.nvim',
    branch = '0.1.x',
    dependencies = {
      'nvim-lua/plenary.nvim',
      { 'nvim-telescope/telescope-fzf-native.nvim', build = 'make' },
    },
    config = function()
      local telescope = require('telescope')
      local actions = require('telescope.actions')
      telescope.setup({
        defaults = {
          layout_strategy = 'vertical',
          layout_config = { preview_cutoff = 20, preview_height = 0.6 },
          mappings = { i = { ['<Esc>'] = actions.close } },
        },
        pickers = {
          find_files = { hidden = true, file_ignore_patterns = { '%.git/' } },
        },
      })
      telescope.load_extension('fzf')
    end,
    keys = {
      { '<D-k>',     function() require('telescope.builtin').find_files() end, desc = 'Find files' },
      { '<D-p>',     function() require('telescope.builtin').commands() end,   desc = 'Command palette' },
      { '<leader>f', function() require('telescope.builtin').find_files() end, desc = 'Find files' },
      { '<leader>b', function() require('telescope.builtin').buffers() end,    desc = 'Find buffers' },
      { '<leader>r', function() require('telescope.builtin').live_grep() end,  desc = 'Ripgrep search' },
      { '<leader>p', function() require('telescope.builtin').commands() end,   desc = 'Command palette' },
      { '<leader>m', function() require('telescope.builtin').keymaps() end,    desc = 'Keybindings' },
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

  -- Multi-cursor (cmd+d select next, cmd+shift+d undo selection)
  {
    'mg979/vim-visual-multi',
    branch = 'master',
    init = function()
      vim.g.VM_maps = {
        ['Find Under']         = '<D-d>',
        ['Find Subword Under'] = '<D-d>',
        ['Remove Region']      = '<D-S-d>',
      }
    end,
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

vim.lsp.config('gopls', {
  cmd = { 'gopls' },
  filetypes = { 'go', 'gomod', 'gowork', 'gotmpl' },
  root_markers = { 'go.work', 'go.mod', '.git' },
})

vim.lsp.enable('gopls')

-- Only gd is mapped here; grn, gra, grr, K are Neovim 0.11+ built-in LSP defaults
vim.api.nvim_create_autocmd('LspAttach', {
  group = augroup,
  callback = function(args)
    vim.keymap.set('n', 'gd', vim.lsp.buf.definition, {
      buffer = args.buf,
      desc = 'Go to definition',
    })
    vim.keymap.set('n', '<C-Space>', vim.lsp.buf.hover, {
      buffer = args.buf,
      desc = 'Hover documentation',
    })
  end,
})
