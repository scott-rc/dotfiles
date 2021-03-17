" ============================================================================
" Settings
" ============================================================================

set nocompatible               " be improved
set hidden                     " allow closing buffers without saving
set encoding=utf8              " use utf8
set ignorecase                 " make searching case insensitive
set smartcase                  " ...unless the query has capital letters.
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
set undodir=~/.vimdid          " ...and store them here
set history=500                " how many lines of history to remember
set wildmenu                   " use wild menu
set backspace=eol,start,indent " make backspace act normal
set autoread                   " auto read when a file is changed from the outside
set ruler                      " show current position
set incsearch                  " makes search act like search in modern browsers
set lazyredraw                 " don't redraw while executing macros (good performance config)
set whichwrap+=<,>,h,l         " go to previous/next line when moving left/right
set t_vb=                      " don't flash the screen
set noerrorbells               " ...and no annoying sound on errors
set novisualbell               " ...none
set nobackup                   " turn backup off
set nowb                       " ...since most stuff is in git anyway
set noswapfile                 " ...right?
set so=4                       " pad 4 lines when moving vertically using j/k

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
" Normal Mode
" ============================================================================

nmap fd <Esc>
nmap ; :
nmap <S-h> ^
nmap <S-l> $
nmap <leader>q :q<CR>
nmap <leader>w :w<CR>
nmap <leader>k <C-w>k
nmap <leader>j <C-w>j
nmap <leader>l <C-w>l
nmap <leader>h <C-w>h
nmap <leader>- :split<CR>
nmap <leader>\ :vsplit<CR>
nmap <leader>[ <C-^>
nmap <leader>] <C-^>

" ============================================================================
" Visual Mode
" ============================================================================

vmap fd <Esc>
vmap <S-h> ^
vmap <S-l> $
vmap <leader>y "+y
vmap J :move '>+1<CR>gv-gv
vmap K :move '<-2<CR>gv-gv

" ============================================================================
" Insert Mode
" ============================================================================

imap fd <Esc>

" ============================================================================
" Command Mode
" ============================================================================
" https://vi.stackexchange.com/questions/22627/switching-arrow-key-mappings-for-wildmenu-tab-completion
set wildcharm=<C-Z>
cmap <expr> <up> wildmenumode() ? "\<left>" : "\<up>"
cmap <expr> <down> wildmenumode() ? "\<right>" : "\<down>"
cmap <expr> <left> wildmenumode() ? "\<up>" : "\<left>"
cmap <expr> <right> wildmenumode() ? " \<bs>\<C-Z>" : "\<right>"

