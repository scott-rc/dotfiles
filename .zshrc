# vim:fileencoding=utf-8:ft=zsh:foldmethod=marker

#: Path {{{

typeset -U path

path=(
    "$HOME/bin"                          # personal
    "/usr/local/bin"                     # homebrew
    "$HOME/.cargo/bin"                   # rust
    "$HOME/.dotnet/tools"                # csharp
    "./node_modules/.bin"                # node
    "$HOME/.symfony/bin" "/usr/local/opt/php@7.1/bin" "/usr/local/opt/php@7.1/sbin" # php
    $path
)

#: }}}

#: Exports {{{

export PATH                              # path
export SSH_KEY_PATH=~/.ssh/rsa_id        # path to ssh
export ZSH="$HOME/.oh-my-zsh"            # path to oh-my-zsh installation.
export GOPATH="$HOME/.go"                # go stuff
export LANG=en_US.UTF-8                  # language environment
export ZDOTDIR="$HOME/.zsh"

if [[ -n $SSH_CONNECTION ]]; then        # editor
  export EDITOR='vim'
else
  export EDITOR='nvim'
fi

#: }}}

#: Settings {{{

setopt extendedglob

HIST_STAMPS="yyyy-mm-dd"                 # time stamp shown in the history command output.

#: }}}

#: Theme {{{

eval "$(starship init zsh)"

#: }}}

#: Plugins {{{

plugins=(zsh-syntax-highlighting docker docker-compose)

source $ZSH/oh-my-zsh.sh

# gcloud completions
source '/usr/local/Caskroom/google-cloud-sdk/latest/google-cloud-sdk/path.zsh.inc'
source '/usr/local/Caskroom/google-cloud-sdk/latest/google-cloud-sdk/completion.zsh.inc'

# kubectl completions
source <(kubectl completion zsh)

#: }}}

#: Key Bindings {{{

bindkey -M menuselect 'h' vi-backward-char
bindkey -M menuselect 'k' vi-up-line-or-history
bindkey -M menuselect 'l' vi-forward-char
bindkey -M menuselect 'j' vi-down-line-or-history

#: }}}

#: Aliases {{{

#: LS / EXA {{{

alias l="exa --all --long --header --git"
alias ls="exa --grid"
alias la="exa --all --long --header --git"

alias ll="ls -lAh"
alias lls="ls -G"
alias lla="ls -lAh"

#: }}}

#: CAT / BAT {{{

alias c=bat
alias cat=bat
alias ccat=cat

#}}}

#: FIND / FD {{{

alias f=fd
alias find=fd
alias ffind=find

#}}}

#: GREP / RIPGREP {{{

alias r=rg
alias grep=rg
alias ggrep=grep

#}}}

#: Vim {{{

alias v="nvim"
alias vim="nvim"

#: }}}

#: Git {{{

alias g="git"
alias ga="git add"
alias gaa="git add --all"
alias gb="git branch"
alias gc!="git commit --amend --no-edit"
alias gc="git commit"
alias gca!="git commit -a --amend --no-edit"
alias gca="git commit -a"
alias gcam!="git commit -a --amend"
alias gcam="git commit -am"
alias gcl="git clean"
alias gcm!="git commit --amend --no-edit"
alias gcm="git commit -m"
alias gco="git checkout"
alias gcob="git checkout -b"
alias gd="git diff"
alias gds="git diff --staged"
alias gl='git log --pretty=oneline --abbrev-commit'
alias gnah="git reset --hard HEAD"
alias gp="git pull"
alias gps="git push"
alias gr="git reset"
alias grh="git reset --hard"
alias grs="git reset --soft"
alias gs="git status -sb"
alias gst="git stash"
alias gstp="git stash pop"
alias gsts="git stash save"
alias gwip!="git add --all && git commit -a --amend --no-edit"
alias gwip="git add --all && git commit -am 'WIP'"

#: }}}

#: JavaScript {{{

alias y="yarn"
alias ya="yarn add"
alias yad="yarn add -D"
alias yb="yarn build"
alias yc="yarn clean"
alias yd="yarn dev"
alias yi="yarn install"
alias yl="yarn lint"
alias yr="yarn run"
alias yrm="yarn remove"
alias ys="yarn start"
alias yt="yarn test"

#: }}}

#: Rust {{{

alias cg="cargo"
alias cgc="cargo check"
alias cgb="cargo build"
alias cgbr="cargo build --release"
alias cgr="cargo run"
alias cgw="cargo-watch -c"
alias cgwr="cargo-watch -c -x run"

cgt() {
    if [ $# -eq 0 ]; then
        cargo test
    else
        cargo test $@
    fi
}

cgwt() {
    if [ $# -eq 0 ]; then
        cw -x test
    else
        cw -x "'test $@'"
    fi
}

#: }}}

#: CSharp {{{

alias dn="dotnet"
alias dna="dotnet add"
alias dnap="dotnet add package"
alias dnb="dotnet build"
alias dnc="dotnet clean"
alias dnefd="dotnet ef database"
alias dnefdu="dotnet ef database update"
alias dnefm="dotnet ef migrations"
alias dnefma="dotnet ef migrations add"
alias dnefmr="dotnet ef migrations remove"
alias dnefmra="dotnet ef migrations remove && dotnet ef migrations add"
alias dnr="dotnet run"
alias dnrm="dotnet remove"
alias dnrmp="dotnet remove package"
alias dnrp="dotnet run --project"
alias dnt="dotnet test --nologo"
alias dnw="dotnet watch"
alias dnwr="dotnet watch run"
alias dnwt="dotnet watch test"

#: }}}

#: Docker {{{

alias d="docker"
alias db="docker build"
alias de="docker exec"
alias dei="docker exec -it"
alias dl="docker logs"
alias dlf="docker logs -f"
alias dp="docker ps"
alias dpa="docker ps -a"
alias dpaq="docker ps -aq"
alias dr="docker run"
alias drm="docker rm"
alias drma='docker rm "`docker ps -a -q`"'
alias ds="docker start"
alias dst="docker stop"
alias dsta='docker stop "`docker ps -a -q`"'

alias dc="docker-compose"
alias dcu="docker-compose up -d"
alias dcd="docker-compose down"

#: }}}

#: Configs {{{

alias zshconf="nvim ~/.zshrc"
alias rzshconf="source ~/.zshrc"
alias vimconf="nvim ~/.vimrc"
alias nvimconf="nvim ~/.config/nvim/init.vim"
alias tmuxconf="nvim ~/.tmux.conf"
alias kittyconf="nvim ~/.config/kitty/kitty.conf"

#: }}}

#: }}}
