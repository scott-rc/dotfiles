# PATH
typeset -U path

path=(
    "$HOME/bin"           # default
    "/usr/local/bin"      # homebrew
    "$HOME/.cargo/bin"    # rust
    "$HOME/.dotnet/tools" # csharp
    "$HOME/.symfony/bin" "/usr/local/opt/php@7.1/bin" "/usr/local/opt/php@7.1/sbin" # php
    $path
)

export PATH
export SSH_KEY_PATH="~/.ssh/rsa_id"      # ssh
export ZSH="$HOME/.oh-my-zsh"            # Path to your oh-my-zsh installation.

HIST_STAMPS="yyyy-mm-dd"                 # time stamp shown in the history command output.
ZSH_THEME="robbyrussell"                 # theme

plugins=()

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

alias g="git"
alias ga="git add"
alias gaa="git add --all"
alias gb="git branch"
alias gc="git commit"
alias gca="git commit -a"
alias gcam="git commit -a -m"
alias gco="git checkout"
alias gd="git diff"
alias gds="git diff --staged"
alias gp="git pull"
alias gps="git push"
alias gs="git status -sb"
alias l8r="git stash"
alias nah="git reset --hard HEAD"
alias pop="git stash pop"
alias wip="git add --all && git commit -a -m 'WIP'"

# js

alias y="yarn"
alias yi="yarn install"
alias ya="yarn add"
alias yad="yarn add -D"

# rust

alias c="cargo"
alias cc="cargo check"
alias cb="cargo build"
alias cbr="cargo build --release"
alias cr="cargo run"
alias cw="cargo-watch -c"
alias cwr="cargo-watch -c -x run"

ct() {
    if [ $# -eq 0 ]; then
        cargo test
    else
        cargo test $@
    fi
}

cwt() {
    if [ $# -eq 0 ]; then
        cw -x test
    else
        cw -x "'test $@'"
    fi
}

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
