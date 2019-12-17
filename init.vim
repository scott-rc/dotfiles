set runtimepath^=~/.vim runtimepath+=~/.vim/after
let &packpath = &runtimepath
source ~/.vimrc

call plug#begin('~/.vim/plugged')
Plug '/usr/local/opt/fzf'
Plug 'Yggdroot/indentLine'
Plug 'airblade/vim-rooter'
Plug 'drewtempelmeyer/palenight.vim'
Plug 'easymotion/vim-easymotion'
Plug 'ervandew/supertab'
Plug 'jiangmiao/auto-pairs'
Plug 'junegunn/fzf.vim'
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
nmap <C-p> :Files<CR>
nmap <leader>b :Buffers<CR>

" ============================================================================
" vim-easymotion
" ============================================================================
map <leader><leader>l <Plug>(easymotion-lineforward)
map <leader><leader>j <Plug>(easymotion-j)
map <leader><leader>k <Plug>(easymotion-k)
map <leader><leader>h <Plug>(easymotion-linebackward)
map / <Plug>(easymotion-sn)
omap / <Plug>(easymotion-tn)
map n <Plug>(easymotion-next)
map N <Plug>(easymotion-prev)

" ============================================================================
" nerdcommenter
" ============================================================================
let g:NERDSpaceDelims = 1
let g:NERDCompactSexyComs = 1
let g:NERDDefaultAlign = 'left'
let g:NERDCommentEmptyLines = 1
let g:NERDTrimTrailingWhitespace = 1
let g:NERDToggleCheckAllLines = 1
map <leader>cc <plug>NERDCommenterToggle
vmap <leader>cc <plug>NERDCommenterToggle

" ============================================================================
" nerdtree
" ============================================================================
map <leader>p :NERDTreeToggle<CR>

" ============================================================================
" supertab
" ============================================================================
let g:SuperTabDefaultCompletionType = "<c-n>"

" ============================================================================
" auto-pairs
" ============================================================================
let g:AutoPairsShortcutToggle = ''
