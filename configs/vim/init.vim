source ~/.vimrc

let data_dir = stdpath('data') . '/site/autoload/plug.vim'
if empty(glob(data_dir))
  silent execute '!curl -fLo ' . data_dir . ' --create-dirs https://raw.githubusercontent.com/junegunn/vim-plug/master/plug.vim'
  autocmd VimEnter * PlugInstall --sync | source ~/.vimrc
endif

call plug#begin('~/.vim/plugged')
Plug 'dag/vim-fish'
Plug 'ervandew/supertab'
Plug 'spinks/vim-leader-guide'
Plug 'tpope/vim-commentary'
call plug#end()

nmap <leader>/ <Plug>CommentaryLine
vmap <leader>/ <Plug>Commentary

nnoremap <silent> <leader> :<c-u>LeaderGuide '<Space>'<CR>
vnoremap <silent> <leader> :<c-u>LeaderGuideVisual '<Space>'<CR>
