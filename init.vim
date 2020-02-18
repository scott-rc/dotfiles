set runtimepath^=~/.vim runtimepath+=~/.vim/after
let &packpath = &runtimepath
source ~/.vimrc

call plug#begin('~/.vim/plugged')
Plug '/usr/local/opt/fzf'
Plug 'Yggdroot/indentLine'
Plug 'airblade/vim-rooter'
Plug 'drewtempelmeyer/palenight.vim'
Plug 'ervandew/supertab'
Plug 'jiangmiao/auto-pairs'
Plug 'junegunn/fzf.vim'
Plug 'neoclide/coc.nvim', {'branch': 'release'}
Plug 'scrooloose/nerdcommenter'
Plug 'scrooloose/nerdtree'
Plug 'vim-airline/vim-airline'
Plug 'vim-airline/vim-airline-themes'
call plug#end()

if has("nvim")
  let $NVIM_TUI_ENABLE_TRUE_COLOR=1
endif

if has("termguicolors")
  set termguicolors
endif

set background=dark
colorscheme palenight
let g:airline_theme = 'palenight'
let g:palenight_terminal_italics=1

" ============================================================================
" airline
" ============================================================================
let g:airline_powerline_fonts = 1

" ============================================================================
" fzf
" ============================================================================
nmap <leader>p :Files<CR>
nmap <leader>b :Buffers<CR>

" ============================================================================
" nerdcommenter
" ============================================================================
let g:NERDSpaceDelims = 1
let g:NERDCompactSexyComs = 1
let g:NERDDefaultAlign = 'left'
let g:NERDCommentEmptyLines = 1
let g:NERDTrimTrailingWhitespace = 1
let g:NERDToggleCheckAllLines = 1
map <leader>/ <plug>NERDCommenterToggle
vmap <leader>/ <plug>NERDCommenterToggle

" ============================================================================
" nerdtree
" ============================================================================
map <leader>1 :NERDTreeToggle<CR>

" ============================================================================
" supertab
" ============================================================================
let g:SuperTabDefaultCompletionType = "<c-n>"

" ============================================================================
" auto-pairs
" ============================================================================
let g:AutoPairsShortcutToggle = ''

" ============================================================================
" coc
" ============================================================================
inoremap <silent><expr> <c-space> coc#refresh()

