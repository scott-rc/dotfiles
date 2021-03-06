" ============================================================================
" Settings
" ============================================================================

set nocompatible               " be improved
set hidden                     " allow closing buffers without saving
set encoding=utf8              " use utf8
set ignorecase                 " make searching case insensitive
set smartcase                  " ... unless the query has capital letters.
set gdefault                   " use 'g' flag by default with :s/foo/bar/.
set splitbelow                 " new windows to the bottom
set splitright                 " new windows to the right
set showmatch                  " show matching brackets.
set number                     " show the line numbers on the left side.
set formatoptions+=o           " continue comment marker in new lines.
set expandtab                  " insert spaces when TAB is pressed.
set tabstop=4                  " render TABs using this many spaces.
set shiftwidth=4               " indentation amount for < and > commands.
set autoindent                 " auto indent new lines
set nowrap                     " don't wrap
set nojoinspaces               " trim whitespace when joining lines
set mouse=n                    " allow using mouse in normal mode
set updatetime=750             " how long to wait before writing swap file
set undofile                   " use undo file
set undodir=~/.vimdid          " ... and store them here
set history=500                " how many lines of history to remember
set wildmenu                   " use wild menu
set backspace=eol,start,indent " make backspace act normal
set autoread                   " auto read when a file is changed from the outside
set ruler                      " show current position
set incsearch                  " makes search act like search in modern browsers
set lazyredraw                 " don't redraw while executing macros (good performance config)
set whichwrap+=<,>,h,l         " go to previous/next line when moving left/right
set t_vb=                      " don't flash the screen
set noerrorbells               " ... and no annoying sound on errors
set novisualbell               " ... none
set nobackup                   " turn backup off
set nowb                       " since most stuff is in git anyway
set noswapfile                 " ...right?

" extra options when running in GUI mode
if has("gui_running")
    set guioptions-=T
    set guioptions-=e
    set t_Co=256
    set guitablabel=%M\ %t
endif

" enable syntax highlighting
syntax enable

" trigger autoload when changing buffers
au FocusGained,BufEnter * checktime

" enable filetype plugins
filetype plugin on
filetype indent on

" ============================================================================
" Globals
" ============================================================================

let mapleader = " "
let g:vim_json_conceal = 0

" ============================================================================
" Commands
" ============================================================================

nmap ; :

nmap <leader>y "+y
vmap <leader>y "+y

" ============================================================================
" Movement
" ============================================================================

" Insert mode
inoremap <C-k> <Up>
inoremap <C-j> <Down>
inoremap <C-l> <Right>
inoremap <C-h> <Left>

" Normal mode
noremap <S-h> ^
noremap <S-l> $

" ============================================================================
" Windows
" ============================================================================

nnoremap <leader>k <C-w>k
nnoremap <leader>j <C-w>j
nnoremap <leader>l <C-w>l
nnoremap <leader>h <C-w>h
nmap <leader>- :split<CR>
nmap <leader>\ :vsplit<CR>
nmap <leader>q :q<CR>
nmap <leader>w :w<CR>
nmap <leader>s :w<CR>
nmap <leader><leader> :e#<CR>

" ============================================================================
" Tabs
" ============================================================================

nmap <leader>t :tabnew<CR>

" ============================================================================
" Editing
" ============================================================================

" Escape
nmap fd <Esc>
imap fd <Esc>
vmap fd <Esc>
omap fd <Esc>

inoremap <expr><C-j> <C-n>
inoremap <expr><C-k> <C-p>

