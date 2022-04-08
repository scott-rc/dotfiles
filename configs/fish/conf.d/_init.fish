/opt/homebrew/bin/brew shellenv | source

# _brew_install exa
# _brew_install fd
# _brew_install jo
# _brew_install jq
# _brew_install rg ripgrep
# _brew_install shellcheck

# #: bat / cat {{{

# alias cat="bat"

# #}}}

# #: btm / top {{{

# alias top="btm"

# #}}}

# #: zoxie / cd {{{

# alias cd="z"
# alias cdi="zi"

# #}}}

# #: docker {{{

# alias d="docker"
# alias db="docker build"
# alias dc="docker compose"
# alias dcd="docker compose down"
# alias dcu="docker compose up -d"
# alias de="docker exec"
# alias dei="docker exec -it"
# alias di="docker image"
# alias dis="docker images"
# alias dl="docker logs"
# alias dlf="docker logs -f"
# alias dp="docker ps"
# alias dpa="docker ps -a"
# alias dpaq="docker ps -aq"
# alias dr="docker run"
# alias drm="docker rm"
# alias drma='docker rm "$(docker ps -a -q)"'
# alias ds="docker start"
# alias dst="docker stop"
# alias dsta='docker stop "$(docker ps -a -q)"'

# #: }}}

# #: exa / ls {{{

# alias l="exa"
# alias ls="exa --all --long --header --git"
# alias lsg="exa --all --long --header --git --grid"

# #: }}}

# #: fd / find {{{

# alias find=fd

# #}}}

# #: go {{{

# alias gob="go build"
# alias gog="go generate"
# alias gor="go run"

# #}}}

# #: js {{{

# alias ni="npm install"
# alias nid="npm install --save-dev"
# alias nr="npm run"
# alias nrb="npm run build"
# alias nrcl="npm run clean"
# alias nrc="npm run check"
# alias nrd="npm run dev"
# alias nrdp="npm run deploy"
# alias nrf="npm run fmt"
# alias nrl="npm run lint"
# alias nrs="npm run start"
# alias nrt="npm run test"
# alias nx="npx"

# alias pi="pnpm install"
# alias pid="pnpm install --save-dev"
# alias pr="pnpm run"
# alias prb="pnpm run build"
# alias prcl="pnpm run clean"
# alias prc="pnpm run check"
# alias prd="pnpm run dev"
# alias prdp="pnpm run deploy"
# alias prf="pnpm run fmt"
# alias prl="pnpm run lint"
# alias prs="pnpm run start"
# alias prt="pnpm run test"
# alias px="pnpx"

# alias y="yarn"
# alias ya="yarn add"
# alias yad="yarn add -D"
# alias yb="yarn build"
# alias yc="yarn clean"
# alias yd="yarn dev"
# alias yi="yarn install"
# alias yl="yarn lint"
# alias yr="yarn run"
# alias yrm="yarn remove"
# alias ys="yarn start"
# alias yt="yarn test"
# alias yu="yarn upgrade-interactive --latest"
# alias yw="yarn workspace"
# alias yws="yarn workspaces"

# #: }}}

# #: ripgrep / grep {{{

# alias grep=rg

# #}}}

# #: rust {{{

# alias c="cargo"
# alias cc="cargo check"
# alias ccc="cargo check; cargo clippy"
# alias cb="cargo build"
# alias cbr="cargo build --release"
# alias cr="cargo run"
# alias crr="cargo run --release"
# alias cw="cargo-watch -c"
# alias cwr="cargo-watch -c -x run"

# #: }}}

# #: terraform {{{

# alias tf="terraform"
# alias tfa="terraform apply"
# alias tfd="terraform destroy"
# alias tfi="terraform init"
# alias tfp="terraform plan"
# alias tfr="terraform refresh"

# #: }}}

# #: vim {{{

# alias v="nvim"
# alias vim="nvim"

# #: }}}
