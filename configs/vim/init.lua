-- ============================================================================
-- Settings
-- ============================================================================

vim.g.loaded_netrw = 1
vim.g.loaded_netrwPlugin = 1
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
vim.o.splitkeep = 'screen'

-- Display
vim.o.title = true
vim.o.titlestring = ' %f%( %m%)'
vim.o.showmode = false
vim.o.showmatch = true
vim.o.number = true
vim.o.wrap = false
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
-- Cmd+Backspace: Ghostty sends \x15 (Ctrl-U) which already does delete-to-beginning in insert mode

-- Navigate by display lines when no count is given (for wrapped lines)
vim.keymap.set({ 'n', 'v' }, 'j', function() return vim.v.count == 0 and 'gj' or 'j' end, { expr = true, desc = 'Down (wrap-aware)' })
vim.keymap.set({ 'n', 'v' }, 'k', function() return vim.v.count == 0 and 'gk' or 'k' end, { expr = true, desc = 'Up (wrap-aware)' })

-- Navigation
vim.keymap.set({ 'n', 'v' }, ';', ':', { desc = 'Command mode' })
vim.keymap.set({ 'n', 'v' }, '<S-h>', '^', { desc = 'Start of line' })
vim.keymap.set({ 'n', 'v' }, '<S-l>', '$', { desc = 'End of line' })

-- Window navigation
vim.keymap.set('n', '<C-h>', '<C-w>h', { desc = 'Focus left window' })
vim.keymap.set('n', '<C-j>', '<C-w>j', { desc = 'Focus below window' })
vim.keymap.set('n', '<C-k>', '<C-w>k', { desc = 'Focus above window' })
vim.keymap.set('n', '<C-l>', '<C-w>l', { desc = 'Focus right window' })

-- Leader
vim.keymap.set({ 'n', 'v' }, '<leader>w', '<cmd>q<CR>', { desc = 'Close' })
vim.keymap.set({ 'n', 'v' }, '<leader>q', '<cmd>qa<CR>', { desc = 'Quit all' })
vim.keymap.set({ 'n', 'v' }, '<leader>s', '<cmd>w<CR>', { desc = 'Save' })
vim.keymap.set('n', '<leader>t', '<cmd>tabnew<CR>', { desc = 'New tab' })
vim.keymap.set('n', '<leader>[', '<cmd>tabp<CR>', { desc = 'Previous tab' })
vim.keymap.set('n', '<leader>]', '<cmd>tabn<CR>', { desc = 'Next tab' })
vim.keymap.set('n', '<leader>%', '<cmd>source %<CR>', { desc = 'Source file' })
vim.keymap.set('v', '<D-c>', '"+y', { desc = 'Copy to clipboard' })
vim.keymap.set({ 'n', 'v', 'i' }, '<D-q>', '<cmd>qa<CR>', { desc = 'Quit' })
vim.keymap.set({ 'n', 'v', 'i' }, '<D-s>', '<cmd>w<CR>', { desc = 'Save' })
vim.keymap.set({ 'n', 'v', 'i' }, '<D-z>', '<cmd>undo<CR>', { desc = 'Undo' })
vim.keymap.set({ 'n', 'v', 'i' }, '<D-S-z>', '<cmd>redo<CR>', { desc = 'Redo' })
vim.keymap.set({ 'n', 'v', 'i' }, '<D-w>', '<cmd>bdelete<CR>', { desc = 'Close buffer' })
vim.keymap.set({ 'n', 'v' }, '<D-[>', '<C-o>', { desc = 'Go back' })
vim.keymap.set({ 'n', 'v' }, '<D-]>', '<C-i>', { desc = 'Go forward' })
vim.keymap.set('i', '<D-[>', '<Esc><C-o>', { desc = 'Go back' })
vim.keymap.set('i', '<D-]>', '<Esc><C-i>', { desc = 'Go forward' })
vim.keymap.set({ 'n', 'v', 'i' }, '<D-1>', function()
  local win = vim.g._last_file_win
  if win and vim.api.nvim_win_is_valid(win) then
    vim.api.nvim_set_current_win(win)
    return
  end
  for _, w in ipairs(vim.api.nvim_list_wins()) do
    local buf = vim.api.nvim_win_get_buf(w)
    if vim.bo[buf].buftype == '' then
      vim.api.nvim_set_current_win(w)
      return
    end
  end
end, { desc = 'Focus primary buffer' })

vim.keymap.set('n', '<leader>yp', function() vim.fn.setreg('+', vim.fn.fnamemodify(vim.fn.expand('%'), ':.')) end, { desc = 'Copy relative path' })
vim.keymap.set('n', '<leader>yP', function() vim.fn.setreg('+', vim.fn.expand('%:p')) end, { desc = 'Copy absolute path' })
vim.keymap.set('n', '<leader>yl', function() vim.fn.setreg('+', vim.fn.fnamemodify(vim.fn.expand('%'), ':.') .. ':' .. vim.fn.line('.')) end, { desc = 'Copy relative path:line' })
vim.keymap.set('n', '<leader>yL', function() vim.fn.setreg('+', vim.fn.expand('%:p') .. ':' .. vim.fn.line('.')) end, { desc = 'Copy absolute path:line' })
vim.keymap.set('v', '<leader>yp', function() vim.fn.setreg('+', vim.fn.fnamemodify(vim.fn.expand('%'), ':.')) end, { desc = 'Copy relative path' })
vim.keymap.set('v', '<leader>yP', function() vim.fn.setreg('+', vim.fn.expand('%:p')) end, { desc = 'Copy absolute path' })
vim.keymap.set('v', '<leader>yl', function()
  local s, e = vim.fn.line('v'), vim.fn.line('.')
  if s > e then s, e = e, s end
  vim.fn.setreg('+', vim.fn.fnamemodify(vim.fn.expand('%'), ':.') .. ':' .. s .. '-' .. e)
end, { desc = 'Copy relative path:lines' })
vim.keymap.set('v', '<leader>yL', function()
  local s, e = vim.fn.line('v'), vim.fn.line('.')
  if s > e then s, e = e, s end
  vim.fn.setreg('+', vim.fn.expand('%:p') .. ':' .. s .. '-' .. e)
end, { desc = 'Copy absolute path:lines' })

local function git_base_branch()
  local branch = vim.fn.systemlist('git rev-parse --abbrev-ref origin/HEAD 2>/dev/null')[1]
  if vim.v.shell_error == 0 and branch and branch ~= '' then
    return (branch:match('origin/(.*)') or branch)
  end
  return 'main'
end

local function github_url(opts)
  local remote = vim.fn.systemlist('git remote get-url origin')[1]
  if vim.v.shell_error ~= 0 or not remote then
    vim.notify('No origin remote', vim.log.levels.WARN)
    return nil
  end
  remote = remote:gsub('git@github%.com:', 'https://github.com/'):gsub('%.git$', '')
  local branch = opts.branch or vim.fn.systemlist('git rev-parse --abbrev-ref HEAD')[1]
  local git_root = vim.fn.systemlist('git rev-parse --show-toplevel')[1]
  local rel_path = vim.fn.expand('%:p'):sub(#git_root + 2)
  local url = remote .. '/blob/' .. branch .. '/' .. rel_path
  if opts.visual then
    local s, e = vim.fn.line('v'), vim.fn.line('.')
    if s > e then s, e = e, s end
    url = url .. '#L' .. s .. '-L' .. e
  end
  return url
end

vim.keymap.set('n', '<leader>go', function()
  local url = github_url({})
  if url then vim.fn.system({ 'open', url }) end
end, { desc = 'Open in GitHub' })
vim.keymap.set('v', '<leader>go', function()
  local url = github_url({ visual = true })
  if url then vim.fn.system({ 'open', url }) end
end, { desc = 'Open in GitHub (selection)' })
vim.keymap.set('n', '<leader>gO', function()
  local url = github_url({ branch = 'main' })
  if url then vim.fn.system({ 'open', url }) end
end, { desc = 'Open in GitHub (main)' })
vim.keymap.set('v', '<leader>gO', function()
  local url = github_url({ branch = 'main', visual = true })
  if url then vim.fn.system({ 'open', url }) end
end, { desc = 'Open in GitHub (main, selection)' })

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

local function close_neotree(source)
  local ei = vim.o.eventignore
  vim.o.eventignore = 'BufEnter,WinEnter,WinLeave,BufLeave'
  require('neo-tree.sources.manager').close(source)
  vim.o.eventignore = ei
end

local function toggle_neotree_files()
  local manager = require('neo-tree.sources.manager')
  local state = manager.get_state('filesystem')
  local neo_win = state.winid
  if neo_win and vim.api.nvim_win_is_valid(neo_win) then
    if vim.api.nvim_get_current_win() == neo_win then
      close_neotree('filesystem')
    else
      vim.api.nvim_set_current_win(neo_win)
    end
  else
    vim.cmd('Neotree focus source=filesystem')
  end
end

vim.api.nvim_create_autocmd({ 'FocusGained', 'BufEnter' }, {
  group = augroup,
  command = 'checktime',
})

-- Track the last window showing a regular file buffer
vim.api.nvim_create_autocmd('BufEnter', {
  group = augroup,
  callback = function()
    if vim.bo.buftype == '' then
      vim.g._last_file_win = vim.api.nvim_get_current_win()
    end
  end,
})

-- Lock neo-tree sidebar width so equalalways doesn't resize it during focus changes
vim.api.nvim_create_autocmd('FileType', {
  group = augroup,
  pattern = 'neo-tree',
  callback = function()
    vim.wo.winfixwidth = true
  end,
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
      require('github-theme').setup({
        groups = {
          all = {
            ['@function.call'] = { style = 'underline' },
            ['@method.call'] = { style = 'underline' },
            ['@comment'] = { style = 'italic' },
          },
        },
      })

      local function is_dark_mode()
        local result = vim.fn.system('defaults read -g AppleInterfaceStyle 2>/dev/null')
        return result:match('Dark') ~= nil
      end

      local function apply_theme()
        local target = is_dark_mode() and 'github_dark_default' or 'github_dark_dimmed'
        if vim.g.colors_name ~= target then
          vim.cmd.colorscheme(target)
        end
      end

      apply_theme()

      -- Re-check on focus so theme follows system appearance changes
      vim.api.nvim_create_autocmd('FocusGained', {
        callback = apply_theme,
      })
    end,
  },

  -- Treesitter (syntax highlighting)
  {
    'nvim-treesitter/nvim-treesitter',
    build = ':TSUpdate',
    opts = {
      ensure_installed = {
        'bash', 'css', 'diff', 'fish', 'go', 'gomod', 'graphql',
        'html', 'javascript', 'json', 'lua', 'markdown',
        'markdown_inline', 'ruby', 'rust', 'toml', 'tsx',
        'typescript', 'vim', 'vimdoc', 'yaml',
      },
    },
  },

  -- Keybinding popup
  {
    'folke/which-key.nvim',
    event = 'VeryLazy',
    opts = {
      preset = 'modern',
      spec = {
        { '<leader>y', group = 'Yank path' },

        { '<leader>g', group = 'Git' },

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
      log_level = 'warn',
      commands = {
        open_and_refocus = function(state)
          local node = state.tree:get_node()
          if node.type == 'directory' then
            require('neo-tree.sources.' .. state.name .. '.commands').open(state)
            return
          end
          local ei = vim.o.eventignore
          vim.o.eventignore = 'BufEnter,WinEnter,WinLeave,BufLeave'
          require('neo-tree.sources.' .. state.name .. '.commands').open(state)
          vim.api.nvim_set_current_win(state.winid)
          vim.o.eventignore = ei
        end,
        open_and_refocus_diff = function(state)
          local node = state.tree:get_node()
          if node.type == 'directory' then
            require('neo-tree.sources.' .. state.name .. '.commands').open(state)
            return
          end
          local ei = vim.o.eventignore
          vim.o.eventignore = 'BufEnter,WinEnter,WinLeave,BufLeave'
          require('neo-tree.sources.' .. state.name .. '.commands').open(state)
          local base = state.git_base or 'HEAD'
          vim.b.diff_base = base
          local diff_win = vim.api.nvim_get_current_win()
          vim.api.nvim_set_current_win(state.winid)
          vim.o.eventignore = ei
          vim.schedule(function()
            vim.api.nvim_win_call(diff_win, function()
              local gs = require('gitsigns')
              gs.change_base(base)
              gs.toggle_deleted(true)
            end)
          end)
        end,
      },
      window = {
        position = 'right',
        mappings = {
          ['<cr>'] = 'open_and_refocus',
          ['<space>'] = 'open_and_refocus',
        },
      },
      filesystem = {
        hijack_netrw_behavior = 'open_default',
        follow_current_file = { enabled = true },
        use_libuv_file_watcher = true,
      },
      git_status = {
        window = {
          mappings = {
            ['<cr>'] = 'open_and_refocus_diff',
            ['<space>'] = 'open_and_refocus_diff',
          },
        },
      },
    },
    keys = {
      { '<leader>e', toggle_neotree_files, desc = 'File explorer' },
      { '<C-e>', toggle_neotree_files, desc = 'Focus/toggle file explorer' },
      { '<D-e>', toggle_neotree_files, mode = { 'n', 'v', 'i' }, desc = 'Focus/toggle file explorer' },
      {
        '<leader>gc',
        function()
          vim.cmd('Neotree git_status git_base=' .. git_base_branch())
        end,
        desc = 'Changed files vs base branch',
      },
      { '<D-g>', function()
          if vim.bo.filetype == 'neo-tree' and vim.b.neo_tree_source == 'git_status' then
            close_neotree('git_status')
          else
            vim.cmd('Neotree focus source=git_status git_base=' .. git_base_branch())
          end
        end, mode = { 'n', 'v', 'i' }, desc = 'Focus/toggle git changes' },
      { '<C-g>', function()
          if vim.bo.filetype == 'neo-tree' and vim.b.neo_tree_source == 'git_status' then
            close_neotree('git_status')
          else
            vim.cmd('Neotree focus source=git_status git_base=' .. git_base_branch())
          end
        end, desc = 'Focus/toggle git changes' },
    },
  },

  -- Git change indicators
  {
    'lewis6991/gitsigns.nvim',
    opts = {
      on_attach = function(bufnr)
        local base = vim.b[bufnr].diff_base
        if base then
          local gs = require('gitsigns')
          gs.change_base(base)
          gs.toggle_deleted(true)
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
          mappings = {
            i = {
              ['<Esc>'] = actions.close,
              ['<M-BS>'] = function(prompt_bufnr)
                local action_state = require('telescope.actions.state')
                local picker = action_state.get_current_picker(prompt_bufnr)
                local prompt = picker:_get_prompt()
                local cursor_col = vim.api.nvim_win_get_cursor(picker.prompt_win)[2] - #picker.prompt_prefix
                local before = prompt:sub(1, cursor_col)
                local trimmed = before:match('^(.-)%s*%S*$') or ''
                local after = prompt:sub(cursor_col + 1)
                picker:reset_prompt(trimmed .. after)
              end,
              ['<C-u>'] = function(prompt_bufnr)
                local action_state = require('telescope.actions.state')
                local picker = action_state.get_current_picker(prompt_bufnr)
                picker:reset_prompt('')
              end,
            },
          },
        },
        pickers = {
          find_files = { hidden = true, file_ignore_patterns = { '%.git/' } },
        },
      })
      telescope.load_extension('fzf')
    end,
    keys = {
      { '<D-k>',     function() require('telescope.builtin').find_files() end, mode = { 'n', 'v', 'i' }, desc = 'Find files' },
      { '<D-p>',     function() require('telescope.builtin').commands() end,   mode = { 'n', 'v', 'i' }, desc = 'Command palette' },
      { '<leader>f', function() require('telescope.builtin').find_files() end, desc = 'Find files' },
      { '<leader>b', function() require('telescope.builtin').buffers() end,    desc = 'Find buffers' },
      { '<D-f>',     function() require('telescope.builtin').live_grep() end, mode = { 'n', 'v', 'i' }, desc = 'Ripgrep search' },
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
    vim.keymap.set({ 'n', 'v', 'i' }, '<D-b>', function()
      local params = vim.lsp.util.make_position_params()
      vim.lsp.buf_request(0, 'textDocument/definition', params, function(err, result)
        if err then return end
        local defs = result or {}
        if not vim.islist(defs) then defs = { defs } end
        local cur_uri = vim.uri_from_bufnr(0)
        local cur_line = params.position.line
        local cur_col = params.position.character
        local at_def = false
        for _, def in ipairs(defs) do
          local loc = def.targetRange or def.targetSelectionRange or def.range
          local uri = def.targetUri or def.uri or ''
          if uri == cur_uri and loc and loc.start.line == cur_line and loc.start.character <= cur_col and (loc['end'].character >= cur_col or loc['end'].line > cur_line) then
            at_def = true
            break
          end
        end
        if at_def or #defs == 0 then
          vim.lsp.buf.references()
        else
          vim.lsp.util.show_document(defs[1], 'utf-8', { focus = true })
        end
      end)
    end, {
      buffer = args.buf,
      desc = 'Go to definition / references',
    })
    vim.keymap.set('n', '<C-Space>', vim.lsp.buf.hover, {
      buffer = args.buf,
      desc = 'Hover documentation',
    })
  end,
})
