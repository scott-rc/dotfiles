# Brewfile - managed by brew bundle
# Run `brew bundle` to install all packages

# Shell
brew "fish"

# Shell tools
brew "shellcheck"
brew "atuin"
brew "bat"
brew "bottom"
brew "direnv"
brew "fd"
brew "fzf"
brew "lsd"
brew "ripgrep"
brew "starship"
brew "zellij"
brew "zoxide"

# Benchmarking
brew "hyperfine"

# Git
brew "git-delta"
brew "git-spice"

# Languages & runtimes
brew "deno"
# Toolchain setup runs as a postinstall hook (only on install/upgrade), keeping
# apply.sh declarative. rustup is keg-linked so this resolves during brew bundle.
brew "rustup", postinstall: "rustup default stable && rustup target add wasm32-wasip1"
brew "rust-analyzer"
brew "go"
brew "node"
brew "pnpm"
brew "typescript-language-server"

# Cloud & infrastructure
cask "gcloud-cli"
brew "kubectl", link: false  # kubernetes-cli formula
# terraform intentionally omitted: provide it per-repo via nix/direnv instead

# Formatters
brew "oxfmt"
brew "stylua"

# Editors
brew "neovim"
brew "tree-sitter-cli"

# Containers
cask "orbstack"

# VSCode extensions (installed via the `code` CLI).
# Note: `brew bundle cleanup` uninstalls any VSCode extension not listed here.
# Cursor shares most of these but is managed separately (its CLI isn't on PATH).
vscode "aaron-bond.better-comments"
vscode "alefragnani.project-manager"
vscode "bierner.markdown-preview-github-styles"
vscode "bmalehorn.vscode-fish"
vscode "bradlc.vscode-tailwindcss"
vscode "cardinal90.multi-cursor-case-preserve"
vscode "craigmaslowski.erb"
vscode "dbaeumer.vscode-eslint"
vscode "denoland.vscode-deno"
vscode "eamodio.gitlens"
vscode "esbenp.prettier-vscode"
vscode "foxundermoon.shell-format"
vscode "github.copilot"
vscode "github.copilot-chat"
vscode "github.github-vscode-theme"
vscode "github.vscode-github-actions"
vscode "github.vscode-pull-request-github"
vscode "golang.go"
vscode "graphql.vscode-graphql-syntax"
vscode "hashicorp.terraform"
vscode "jnoortheen.nix-ide"
vscode "meganrogge.template-string-converter"
vscode "mkhl.direnv"
vscode "ms-azuretools.vscode-docker"
vscode "ms-kubernetes-tools.vscode-kubernetes-tools"
vscode "openai.openai-chatgpt-adhoc"
vscode "pomdtr.excalidraw-editor"
vscode "redhat.vscode-yaml"
vscode "riey.erb"
vscode "ryuta46.multi-command"
vscode "stkb.rewrap"
vscode "streetsidesoftware.code-spell-checker"
vscode "tim-koehler.helm-intellisense"
vscode "timonwong.shellcheck"
vscode "unifiedjs.vscode-mdx"
vscode "usernamehw.errorlens"
vscode "vortizhe.simple-ruby-erb"
vscode "vscode-icons-team.vscode-icons"
vscode "vscodevim.vim"
vscode "zxh404.vscode-proto3"
