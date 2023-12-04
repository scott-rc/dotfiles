# Fig pre block. Keep at the top of this file.
[[ -f "$HOME/.fig/shell/zshrc.pre.zsh" ]] && . "$HOME/.fig/shell/zshrc.pre.zsh"
# vim:fileencoding=utf-8:ft=zsh:foldmethod=marker

#: Exports {{{

export LANG=en_US.UTF-8

export ZSH="$HOME/.oh-my-zsh"
export HISTFILE=~/.zsh_history
export SSH_KEY_PATH=~/.ssh/rsa_id

export CSHARPPATH="$HOME/.dotnet/tools"
export GOPATH="$HOME/.go"
export HOMEBREWPATH="/opt/homebrew/bin"
export JAVAPATH="/opt/homebrew/opt/openjdk/bin"
export NODEPATH="./node_modules/.bin"
export NVM_DIR="$HOME/.nvm"
export PHPPATH="/opt/homebrew/opt/php@7.4/bin:/opt/homebrew/opt/php@7.4/sbin:$HOME/.symfony/bin"
export RUSTPATH="$HOME/.cargo/bin"

#: }}}

#: Path {{{

typeset -U path

path=(
    "$HOME/bin"
    "$PHPPATH"
    "$HOMEBREWPATH"
    "$JAVAPATH"
    "$RUSTPATH"
    "$NODEPATH"
    "$GOPATH/bin"
    $path
)

export PATH

#: }}}

#: Settings {{{

DISABLE_UPDATE_PROMPT=true # just do it
HIST_STAMPS="yyyy-mm-dd"   # time stamp shown in the history command output.

#: }}}

#: Plugins {{{

# export MCFLY_FUZZY=true
export MCFLY_KEY_SCHEME=vim

eval "$(starship init zsh)"
eval "$(zoxide init zsh)"
eval "$(mcfly init zsh)"

plugins=(
    zsh-syntax-highlighting       # zsh
    fd gitfast                    # utils
    docker docker-compose kubectl # docker
    terraform aws gcloud          # cloud
    nvm npm yarn                  # node
    golang                        # go
)

source $ZSH/oh-my-zsh.sh

[ -s "$NVM_DIR/nvm.sh" ] && \. "$NVM_DIR/nvm.sh"                   # This loads nvm
[ -s "$NVM_DIR/bash_completion" ] && \. "$NVM_DIR/bash_completion" # This loads nvm bash_completion

autoload -U +X bashcompinit && bashcompinit

# test -e "${HOME}/.iterm2_shell_integration.zsh" && source "${HOME}/.iterm2_shell_integration.zsh"

#: }}}

#: Aliases {{{

#: bat / cat {{{

alias _cat="cat"
alias cat="bat"

#}}}

#: btm / top {{{

alias _top="top"
alias top="btm"

#}}}

#: zoxie / cd {{{

alias _cd="cd"
alias cd="z"
alias cdi="zi"

#}}}

#: configs {{{

alias zshconf="nvim ~/.zshrc"
alias rzshconf="source ~/.zshrc"
alias vimconf="nvim ~/.vimrc"
alias nvimconf="nvim ~/.config/nvim/init.vim"
alias awsconf="nvim ~/.aws/credentials"

#: }}}

#: docker {{{

alias d="docker"
alias db="docker build"
alias dc="docker compose"
alias dcd="docker compose down"
alias dcu="docker compose up -d"
alias de="docker exec"
alias dei="docker exec -it"
alias di="docker image"
alias dis="docker images"
alias dl="docker logs"
alias dlf="docker logs -f"
alias dp="docker ps"
alias dpa="docker ps -a"
alias dpaq="docker ps -aq"
alias dr="docker run"
alias drm="docker rm"
alias drma='docker rm "$(docker ps -a -q)"'
alias ds="docker start"
alias dst="docker stop"
alias dsta='docker stop "$(docker ps -a -q)"'

#: }}}

#: exa / ls {{{

alias _ls="ls"
alias l="exa"
alias ls="exa --all --long --header --git"
alias lsg="exa --all --long --header --git --grid"

#: }}}

#: fd / find {{{

alias f=fd
alias find=fd
alias ffind=find

#}}}

#: git {{{

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
alias gd~="git diff HEAD~"
alias gf="git fetch"
alias gl='git log --pretty=oneline --abbrev-commit'
alias gp="git pull"
alias gps="git push"
alias gr="git reset"
alias gr~="git reset HEAD~"
alias grh="git reset --hard"
alias grs="git reset --soft"
alias gs="git status -sb"
alias gst="git stash"
alias gstp="git stash pop"
alias gsts="git stash save"
alias gui="gitui"
alias gwip!="git add --all && git commit -a --amend --no-edit"
alias gwip="git add --all && git commit -am 'WIP'"

#: }}}

#: go {{{

alias gob="go build"
alias gog="go generate"
alias gor="go run"

#}}}

#: js {{{

alias ni="npm install"
alias nid="npm install --save-dev"
alias nr="npm run"
alias nrb="npm run build"
alias nrcl="npm run clean"
alias nrc="npm run check"
alias nrd="npm run dev"
alias nrdp="npm run deploy"
alias nrf="npm run fmt"
alias nrl="npm run lint"
alias nrs="npm run start"
alias nrt="npm run test"
alias nx="npx"

alias pi="pnpm install"
alias pid="pnpm install --save-dev"
alias pr="pnpm run"
alias prb="pnpm run build"
alias prcl="pnpm run clean"
alias prc="pnpm run check"
alias prd="pnpm run dev"
alias prdp="pnpm run deploy"
alias prf="pnpm run fmt"
alias prl="pnpm run lint"
alias prs="pnpm run start"
alias prt="pnpm run test"
alias px="pnpx"

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
alias yu="yarn upgrade-interactive --latest"
alias yw="yarn workspace"
alias yws="yarn workspaces"

#: }}}

#: ripgrep / grep {{{

alias r=rg
alias grep=rg
alias ggrep=grep

#}}}

#: rust {{{

alias c="cargo"
alias cc="cargo check"
alias ccc="cargo check; cargo clippy"
alias cb="cargo build"
alias cbr="cargo build --release"
alias cr="cargo run"
alias crr="cargo run --release"
alias cw="cargo-watch -c"
alias cwr="cargo-watch -c -x run"

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

#: terraform {{{

alias tf="terraform"
alias tfa="terraform apply"
alias tfd="terraform destroy"
alias tfi="terraform init"
alias tfp="terraform plan"
alias tfr="terraform refresh"

#: }}}

#: vim {{{

alias v="nvim"
alias vim="nvim"

#: }}}

#: }}}

# Fig post block. Keep at the bottom of this file.
[[ -f "$HOME/.fig/shell/zshrc.post.zsh" ]] && . "$HOME/.fig/shell/zshrc.post.zsh"
