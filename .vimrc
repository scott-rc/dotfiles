" ============================================================================
" Settings
" ============================================================================

syntax on
set hidden              " A lot of plugins require this
set ignorecase          " Make searching case insensitive
set smartcase           " ... unless the query has capital letters.
set gdefault            " Use 'g' flag by default with :s/foo/bar/.
set splitbelow          " new windows to the bottom
set splitright          " new windows to the right
set showmatch           " Show matching brackets.
set number              " Show the line numbers on the left side.
set formatoptions+=o    " Continue comment marker in new lines.
set expandtab           " Insert spaces when TAB is pressed.
set tabstop=4           " Render TABs using this many spaces.
set shiftwidth=4        " Indentation amount for < and > commands.
set autoindent
set nowrap
set nojoinspaces
set mouse=n
set updatetime=100
set undodir=~/.vimdid
set undofile

" ============================================================================
" Globals
" ============================================================================

let mapleader = ","

" ============================================================================
" Commands
" ============================================================================

nmap ; :
nmap <leader>R :source ~/.config/nvim/init.vim<CR>

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

nnoremap <M-k> <C-w>k
nnoremap <M-j> <C-w>j
nnoremap <M-l> <C-w>l
nnoremap <M-h> <C-w>h
nmap <leader>- :split<CR>
nmap <leader>\ :vsplit<CR>
nmap <leader>q :q<CR>
nmap <leader>w :w<CR>
nmap <leader><leader> <C-^>

" ============================================================================
" Tabs
" ============================================================================

nmap <leader>t :tabnew<CR>
nmap <leader>1 1gt
nmap <leader>2 2gt
nmap <leader>3 3gt
nmap <leader>4 4gt
nmap <leader>5 5gt
nmap <leader>6 6gt
nmap <leader>7 7gt
nmap <leader>8 8gt
nmap <leader>9 9gt

" ============================================================================
" Editing
" ============================================================================

" Escape
nmap fd <Esc>
imap fd <Esc>
vmap fd <Esc>
omap fd <Esc>
