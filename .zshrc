# vim:fileencoding=utf-8:ft=zsh:foldmethod=marker

#: Path {{{

typeset -U path

path=(
    "$HOME/bin"                          # personal
    "/usr/local/bin"                     # homebrew
    "$HOME/.cargo/bin"                   # rust
    "$HOME/.dotnet/tools"                # csharp
    "./node_modules/.bin"                # node
    "$HOME/.go/bin"                      # go
    "$HOME/.symfony/bin" "/usr/local/opt/php@7.3/bin" "/usr/local/opt/php@7.3/sbin" # php
    $path
)

#: }}}

#: Exports {{{

export PATH
export SSH_KEY_PATH=~/.ssh/rsa_id
export ZSH="$HOME/.oh-my-zsh"
export GOPATH="$HOME/.go"
export NVM_DIR="$HOME/.nvm"
export LANG=en_US.UTF-8

#: }}}

#: Settings {{{

DISABLE_UPDATE_PROMPT=true               # just do it
HIST_STAMPS="yyyy-mm-dd"                 # time stamp shown in the history command output.

#: }}}

#: Prompt {{{

eval "$(starship init zsh)"

#: }}}

#: Plugins {{{

plugins=(
    zsh-syntax-highlighting        # zsh
    fasd fd gitfast                # utils
    docker docker-compose kubectl  # docker
    terraform aws gcloud           # cloud
    nvm npm yarn                   # node
    golang                         # go
)

source $ZSH/oh-my-zsh.sh

#: }}}

#: Aliases {{{

#: BAT / CAT {{{

alias c=bat
alias cat=bat
alias ccat=cat

#}}}

#: BTM / TOP {{{

alias top=btm
alias ttop=top

#}}}

#: Configs {{{

alias zshconf="nvim ~/.zshrc"
alias rzshconf="source ~/.zshrc"
alias vimconf="nvim ~/.vimrc"
alias nvimconf="nvim ~/.config/nvim/init.vim"
alias awsconf="nvim ~/.aws/credentials"

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

#: EXA / LS {{{

alias l="exa --all --long --header --git"
alias ls="exa --grid"
alias la="exa --all --long --header --git"

alias ll="ls -lAh"
alias lls="ls -G"
alias lla="ls -lAh"

#: }}}

#: FD / FIND {{{

alias f=fd
alias find=fd
alias ffind=find

#}}}

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
alias gcm!="git commit --amend"
alias gcm="git commit -m"
alias gco="git checkout"
alias gcob="git checkout -b"
alias gd="git diff"
alias gds="git diff --staged"
alias gf="git fetch"
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

#: Go {{{

alias gob="go build"
alias gog="go generate"
alias gor="go run"

#}}}

#: JavaScript {{{

alias n="npm"
alias nb="npm run build"
alias nc="npm run clean"
alias nd="npm run dev"
alias ni="npm install"
alias nid="npm install --save-dev"
alias nl="npm run lint"
alias nr="npm run"
alias nrm="npm remove"
alias ns="npm run start"
alias nt="npm run test"

alias y="yarn"
alias ya="yarn add"
alias yad="yarn add -D"
alias yb="yarn build"
alias yc="yarn clean"
alias yd="yarn dev"
alias yg="yarn generate"
alias yi="yarn install"
alias yl="yarn lint"
alias yr="yarn run"
alias yrm="yarn remove"
alias ys="yarn start"
alias yt="yarn test"
alias yu="yarn upgrade-interactive --latest"
alias yw="yarn workspace"
alias yws="yarn workspaces"

#: }}}

#: Lazy {{{

alias lzg="lazygit"
alias lzd="lazydocker"

#: }}}

#: RIPGREP / GREP {{{

alias r=rg
alias grep=rg
alias ggrep=grep

#}}}

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

#: Terraform {{{

alias tf="terraform"
alias tfa="terraform apply"
alias tfd="terraform destroy"
alias tfr="terraform refresh"

#: }}}

#: Vim {{{

alias v="nvim"
alias vim="nvim"

#: }}}

#: }}}

autoload -U +X bashcompinit && bashcompinit
complete -o nospace -C /usr/local/bin/bit bit
