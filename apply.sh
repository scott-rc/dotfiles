#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# shellcheck source=lib.sh
source "$SCRIPT_DIR/lib.sh"

# --- Pre-flight Checks ---

if [ -z "${HOME:-}" ]; then
	log_error "HOME environment variable is not set"
	exit 1
fi

defaults write -g ApplePressAndHoldEnabled -bool false
# shellcheck disable=SC2016
defaults write NSGlobalDomain NSUserKeyEquivalents -dict-add "Minimize" '@~^$m'

# Determine the workspace root (assumed to be the directory of this script)
WORKSPACE_ROOT="$SCRIPT_DIR"
CONFIGS="$WORKSPACE_ROOT/configs"

# Ensure $HOME/.config exists
mkdir -p "$HOME/.config"

log_section "Homebrew"

if [ -x "/opt/homebrew/bin/brew" ]; then
	log_debug "Homebrew is already installed"
else
	run_with_spinner "Installing Homebrew" /usr/bin/env -S bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
	run_with_spinner "Updating Homebrew" /opt/homebrew/bin/brew update --force
fi

run_with_spinner "Installing packages from Brewfile" /opt/homebrew/bin/brew bundle --file="$WORKSPACE_ROOT/Brewfile"

log_section "Bat"

ensure_symlink "$CONFIGS/bat" "$HOME/.config/bat"
bat_cache_dir="$(bat --cache-dir 2>/dev/null || echo "")"
if [[ -z "$bat_cache_dir" || ! -d "$bat_cache_dir/syntaxes" || "$CONFIGS/bat" -nt "$bat_cache_dir/syntaxes" ]]; then
	run_with_spinner "Rebuilding bat cache" bat cache --build
else
	log_success "Cache up to date"
fi

log_section "Fish Shell"

ensure_symlink "$CONFIGS/fish" "$HOME/.config/fish"

fish_changed=false
if grep -q "/opt/homebrew/bin/fish" /etc/shells; then
	log_debug "Fish is already added to /etc/shells"
else
	log_info "Adding fish to /etc/shells"
	echo "/opt/homebrew/bin/fish" | sudo tee -a /etc/shells >/dev/null
	fish_changed=true
fi

# shellcheck disable=SC2016
if /opt/homebrew/bin/fish -c 'contains /opt/homebrew/bin $fish_user_paths'; then
	log_debug "Homebrew is already in fish_user_paths"
else
	log_info "Adding Homebrew to fish_user_paths"
	# shellcheck disable=SC2016
	/opt/homebrew/bin/fish -c 'set -U fish_user_paths /opt/homebrew/bin $fish_user_paths'
	fish_changed=true
fi

if [[ "$fish_changed" == "false" ]]; then
	log_success "Fish configured"
fi

log_section "Symlinks"

ensure_symlink "$CONFIGS/glow/glow.yml" "$HOME/Library/Preferences/glow/glow.yml"
ensure_symlink "$CONFIGS/ghostty/config" "$HOME/Library/Application Support/com.mitchellh.ghostty/config"
ensure_symlink "$CONFIGS/gitui/github-dark.ron" "$HOME/.config/gitui/theme.ron"
ensure_symlink "$CONFIGS/gitui/github-dark-dimmed.ron" "$HOME/.config/gitui/github-dark-dimmed.ron"
ensure_symlink "$CONFIGS/lsd" "$HOME/.config/lsd"
ensure_symlink "$CONFIGS/lazygit/config.yml" "$HOME/Library/Application Support/lazygit/config.yml"
ensure_symlink "$CONFIGS/git/.gitconfig" "$HOME/.gitconfig"
ensure_symlink "$CONFIGS/git/.gitignore_global" "$HOME/.config/git/.gitignore_global"
ensure_symlink "$CONFIGS/atuin/config.toml" "$HOME/.config/atuin/config.toml"
ensure_symlink "$CONFIGS/direnv/direnv.toml" "$HOME/.config/direnv/direnv.toml"
ensure_symlink "$CONFIGS/karabiner/karabiner.json" "$HOME/.config/karabiner/karabiner.json"
ensure_symlink "$CONFIGS/orbstack/docker.json" "$HOME/.orbstack/config/docker.json"
ensure_symlink "$CONFIGS/starship/starship.toml" "$HOME/.config/starship.toml"
ensure_symlink "$CONFIGS/nvim" "$HOME/.config/nvim"
ensure_symlink "$CONFIGS/zed/settings.json" "$HOME/.config/zed/settings.json"
ensure_symlink "$CONFIGS/zed/keymap.json" "$HOME/.config/zed/keymap.json"
ensure_symlink "$CONFIGS/zellij/config.kdl" "$HOME/.config/zellij/config.kdl"
ensure_symlink "$CONFIGS/zellij/layouts" "$HOME/.config/zellij/layouts"
ensure_symlink "$CONFIGS/zsh/.zshrc" "$HOME/.zshrc"

expected_iterm2_prefs="$CONFIGS/iterm2"
current_iterm2_prefs=$(defaults read com.googlecode.iterm2 PrefsCustomFolder 2>/dev/null || echo "")
if [ "$current_iterm2_prefs" = "$expected_iterm2_prefs" ]; then
	log_debug "iTerm2 is already loading preferences from the correct location"
else
	log_info "Setting iTerm2 preferences folder to $expected_iterm2_prefs"
	defaults write com.googlecode.iterm2 PrefsCustomFolder -string "$expected_iterm2_prefs"
fi

current_load_prefs=$(defaults read com.googlecode.iterm2 LoadPrefsFromCustomFolder 2>/dev/null || echo "")
if [ "$current_load_prefs" = "1" ]; then
	log_debug "iTerm2 is already set to load preferences from the custom folder"
else
	log_info "Enabling iTerm2 to load preferences from the custom folder"
	defaults write com.googlecode.iterm2 LoadPrefsFromCustomFolder -bool true
fi

log_success "Config symlinks"

log_section "Claude Code & Codex"

ensure_symlink "$CONFIGS/claude/settings.json" "$HOME/.claude/settings.json"
ensure_symlink "$CONFIGS/claude/keybindings.json" "$HOME/.claude/keybindings.json"
ensure_symlink "$CONFIGS/claude/commands" "$HOME/.claude/commands"
ensure_symlink "$CONFIGS/claude/skills" "$HOME/.claude/skills"
ensure_symlink "$CONFIGS/claude/hooks" "$HOME/.claude/hooks"
ensure_symlink "$CONFIGS/claude/CLAUDE.md" "$HOME/.claude/CLAUDE.md"
ensure_symlink "$CONFIGS/claude/statusline" "$HOME/.claude/statusline"
ensure_symlink "$CONFIGS/claude/rules" "$HOME/.claude/rules"
ensure_symlink "$CONFIGS/claude/CLAUDE.md" "$HOME/.codex/AGENTS.md"
ensure_symlink "$CONFIGS/codex/config.toml" "$HOME/.codex/config.toml"
ensure_symlink "$CONFIGS/codex/rules/default.rules" "$HOME/.codex/rules/default.rules"
ensure_symlink "$CONFIGS/claude/rules" "$HOME/.codex/claude-rules"

mkdir -p "$HOME/.codex/skills" "$HOME/.agents/skills"
for skill_dir in "$CONFIGS/claude/skills"/*; do
	[ -d "$skill_dir" ] || continue
	skill_name="$(basename "$skill_dir")"
	ensure_symlink "$skill_dir" "$HOME/.codex/skills/$skill_name"
	ensure_symlink "$skill_dir" "$HOME/.agents/skills/$skill_name"
done
log_success "Claude Code & Codex symlinks"

log_section "Cursor"

ensure_symlink "$CONFIGS/cursor/settings.json" "$HOME/Library/Application Support/Cursor/User/settings.json"
ensure_symlink "$CONFIGS/cursor/keybindings.json" "$HOME/Library/Application Support/Cursor/User/keybindings.json"
ensure_symlink "$CONFIGS/cursor/mcp.json" "$HOME/.cursor/mcp.json"
log_success "Cursor symlinks"

log_section "Nix"

nix_changed=false
if [ -f "/nix/var/nix/profiles/default/etc/profile.d/nix-daemon.sh" ]; then
	log_debug "Nix is already installed"
else
	log_info "Installing Nix"
	sh <(curl -L https://nixos.org/nix/install)
	nix_changed=true
fi

ensure_symlink "$CONFIGS/nix/nix.conf" "/etc/nix/nix.conf"

if command -v nixpkgs-fmt &> /dev/null; then
	log_debug "nixpkgs-fmt is already installed"
else
	run_with_spinner "Installing nixpkgs-fmt" nix profile install nixpkgs#nixpkgs-fmt
	nix_changed=true
fi

if [[ "$nix_changed" == "false" ]]; then
	log_success "Nix configured"
fi

log_section "Rust"

RUSTUP_BIN="/opt/homebrew/opt/rustup/bin"
rust_changed=false
if [ -x "$RUSTUP_BIN/rustup" ]; then
	export PATH="$RUSTUP_BIN:$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$PATH"
	if ! rustup toolchain list 2>/dev/null | grep -q "stable"; then
		run_with_spinner "Installing Rust stable toolchain" rustup default stable
		rust_changed=true
	fi
	if ! rustup target list --installed 2>/dev/null | grep -q "wasm32-wasip1"; then
		run_with_spinner "Adding wasm32-wasip1 target" rustup target add wasm32-wasip1
		rust_changed=true
	fi
fi

if [[ "$rust_changed" == "false" ]]; then
	log_success "Rust toolchain ready"
fi

log_section "Tools"

if command -v cargo &>/dev/null; then
	mkdir -p "$HOME/.cargo/bin"
	run_with_spinner "Building tools workspace" bash -c "cd \"$WORKSPACE_ROOT/tools\" && cargo build --release 2>&1"
	ensure_symlink "$WORKSPACE_ROOT/tools/target/release/md" "$HOME/.cargo/bin/md"
	ensure_symlink "$WORKSPACE_ROOT/tools/target/release/gd" "$HOME/.cargo/bin/gd"
	ensure_symlink "$WORKSPACE_ROOT/tools/target/release/boom" "$HOME/.cargo/bin/boom"
	log_success "Tools built and linked"
fi

echo ""
echo -e "${GREEN}✨ Setup complete!${RESET}"
