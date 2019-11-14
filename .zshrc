export PATH=$HOME/bin:/usr/local/bin:$HOME/.cargo/bin:$HOME/.dotnet/tools:$HOME/.symfony/bin:/usr/local/opt/php@7.1/bin:/usr/local/opt/php@7.1/sbin:$PATH
export SSH_KEY_PATH="~/.ssh/rsa_id"      # ssh
export ZSH="$HOME/.oh-my-zsh"            # Path to your oh-my-zsh installation.

ENABLE_CORRECTION="true"                 # command auto-correction.
HIST_STAMPS="yyyy-mm-dd"                 # time stamp shown in the history command output.
ZSH_THEME="robbyrussell"                 # theme

plugins=(git yarn)

source $ZSH/oh-my-zsh.sh

# User configuration

export LANG=en_US.UTF-8 # set your language environment

# Preferred editor for local and remote sessions

if [[ -n $SSH_CONNECTION ]]; then
  export EDITOR='vim'
else
  export EDITOR='nvim'
fi

# Set personal aliases

alias vim="nvim"

# git

alias wip="git add . && git commit -a -m 'WIP'"
alias nah="git reset --hard HEAD"
alias l8r="git stash"
alias pop="git stash pop"

# rust

alias c="cargo"
alias cc="cargo check"
alias cr="cargo run"
alias cw="cargo-watch -c"

# docker

alias d="docker"
alias dc="docker-compose"

# configs

alias zshconf="nvim ~/.zshrc"
alias rzshconf="source ~/.zshrc"
alias vimconf="nvim ~/.vimrc"
alias nvimconf="nvim ~/.config/nvim/init.vim"
alias tmuxconf="nvim ~/.tmux.conf"
alias kittyconf="nvim ~/.config/kitty/kitty.conf"
