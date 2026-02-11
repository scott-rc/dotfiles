#!/usr/bin/env bash

set -euo pipefail

RED="\033[31m"
GREEN="\033[32m"
YELLOW="\033[33m"
CYAN="\033[36m"
RESET="\033[0m"

LOG_LEVEL="${LOG_LEVEL:-info}"

log_debug() {
	if [[ "$LOG_LEVEL" == "debug" ]]; then
		echo -e "${GREEN}[DEBUG]${RESET} $1"
	fi
}

log_info() {
	if [[ "$LOG_LEVEL" == "debug" || "$LOG_LEVEL" == "info" ]]; then
		echo -e "${CYAN}[INFO]${RESET} $1"
	fi
}

log_warn() {
	if [[ "$LOG_LEVEL" == "debug" || "$LOG_LEVEL" == "info" || "$LOG_LEVEL" == "warn" ]]; then
		echo -e "${YELLOW}[WARN]${RESET} $1"
	fi
}

log_error() {
	echo -e "${RED}[ERROR]${RESET} $1" >&2
}

# Create a symlink from TARGET to SOURCE.
# If TARGET already exists as a symlink pointing to SOURCE, do nothing.
# Otherwise, move TARGET to TARGET.bak and create the new symlink.
ensure_symlink() {
	local source="$1"
	local target="$2"

	if [ -L "$target" ]; then
		local current_source
		current_source=$(readlink "$target")
		if [ "$current_source" = "$source" ]; then
			log_debug "Symlink $target already exists and points to $source"
			return
		fi
	fi

	dir="$(dirname "$target")"

	log_info "Creating symlink from $target to $source"
	if [ -e "$target" ]; then
		log_info "Moving existing file $target to $target.bak"
		if [[ "$dir" == /etc/* ]]; then
			sudo mv "$target" "$target.bak"
		else
			mv "$target" "$target.bak"
		fi
	fi

	if [[ "$dir" == /etc/* ]]; then
		sudo mkdir -p "$dir"
		sudo ln -s "$source" "$target"
	else
		mkdir -p "$dir"
		ln -s "$source" "$target"
	fi

}

# --- Pre-flight Checks ---

if [ -z "${HOME:-}" ]; then
	log_error "HOME environment variable is not set"
	exit 1
fi

defaults write -g ApplePressAndHoldEnabled -bool false
defaults write NSGlobalDomain NSUserKeyEquivalents -dict-add "Minimize" '@~^$m'

# Determine the workspace root (assumed to be the directory of this script)
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORKSPACE_ROOT="$SCRIPT_DIR"
CONFIGS="$WORKSPACE_ROOT/configs"

# Ensure $HOME/.config exists
mkdir -p "$HOME/.config"

# --- Homebrew ---

if [ -x "/opt/homebrew/bin/brew" ]; then
	log_debug "Homebrew is already installed"
else
	log_info "Installing Homebrew"
	/usr/bin/env -S bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
	/opt/homebrew/bin/brew update --force
fi

# Install all packages from Brewfile
log_info "Installing packages from Brewfile"
/opt/homebrew/bin/brew bundle --file="$WORKSPACE_ROOT/Brewfile"

# --- Fish Shell ---

# Create a symlink for the fish configuration.
ensure_symlink "$CONFIGS/fish" "$HOME/.config/fish"

# Add fish to /etc/shells if it isn't already there.
if grep -q "/opt/homebrew/bin/fish" /etc/shells; then
	log_debug "Fish is already added to /etc/shells"
else
	log_info "Adding fish to /etc/shells"
	# Note: Writing to /etc/shells may require sudo privileges.
	echo "/opt/homebrew/bin/fish" | sudo tee -a /etc/shells >/dev/null
fi

# Add Homebrew to fish_user_paths (universal variable, persists across sessions)
if /opt/homebrew/bin/fish -c 'contains /opt/homebrew/bin $fish_user_paths'; then
	log_debug "Homebrew is already in fish_user_paths"
else
	log_info "Adding Homebrew to fish_user_paths"
	/opt/homebrew/bin/fish -c 'set -U fish_user_paths /opt/homebrew/bin $fish_user_paths'
fi

# --- Glow ---

ensure_symlink "$CONFIGS/glow/glow.yml" "$HOME/Library/Preferences/glow/glow.yml"

# --- Ghostty ---

ensure_symlink "$CONFIGS/ghostty/config" "$HOME/Library/Application Support/com.mitchellh.ghostty/config"

# --- gitui ---

ensure_symlink "$CONFIGS/gitui/github-dark.ron" "$HOME/.config/gitui/theme.ron"
ensure_symlink "$CONFIGS/gitui/github-dark-dimmed.ron" "$HOME/.config/gitui/github-dark-dimmed.ron"

# --- Lazygit ---

ensure_symlink "$CONFIGS/lazygit/config.yml" "$HOME/Library/Application Support/lazygit/config.yml"

# --- Git ---

ensure_symlink "$CONFIGS/git/.gitconfig" "$HOME/.gitconfig"
ensure_symlink "$CONFIGS/git/.gitignore_global" "$HOME/.config/git/.gitignore_global"

# --- iTerm2 Preferences ---

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

# --- Atuin ---

ensure_symlink "$CONFIGS/atuin/config.toml" "$HOME/.config/atuin/config.toml"

# --- Claude Code ---

ensure_symlink "$CONFIGS/claude/settings.json" "$HOME/.claude/settings.json"
ensure_symlink "$CONFIGS/claude/keybindings.json" "$HOME/.claude/keybindings.json"
ensure_symlink "$CONFIGS/claude/commands" "$HOME/.claude/commands"
ensure_symlink "$CONFIGS/claude/skills" "$HOME/.claude/skills"
ensure_symlink "$CONFIGS/claude/hooks" "$HOME/.claude/hooks"
ensure_symlink "$CONFIGS/claude/CLAUDE.md" "$HOME/.claude/CLAUDE.md"
ensure_symlink "$CONFIGS/claude/starship.toml" "$HOME/.claude/starship.toml"
ensure_symlink "$CONFIGS/claude/starship-claude" "$HOME/.local/bin/starship-claude"

# --- direnv ---

ensure_symlink "$CONFIGS/direnv/direnv.toml" "$HOME/.config/direnv/direnv.toml"

# --- Karabiner ---

ensure_symlink "$CONFIGS/karabiner/karabiner.json" "$HOME/.config/karabiner/karabiner.json"

# --- Nix ---

if [ -f "/nix/var/nix/profiles/default/etc/profile.d/nix-daemon.sh" ]; then
	log_debug "Nix is already installed"
else
	log_info "Installing Nix"
	sh <(curl -L https://nixos.org/nix/install)
fi

ensure_symlink "$CONFIGS/nix/nix.conf" "/etc/nix/nix.conf"

if command -v nixpkgs-fmt &> /dev/null; then
	log_debug "nixpkgs-fmt is already installed"
else
	log_info "Installing nixpkgs-fmt"
	nix profile install nixpkgs#nixpkgs-fmt
fi

# --- Orbstack ---

ensure_symlink "$CONFIGS/orbstack/docker.json" "$HOME/.orbstack/config/docker.json"

# --- Starship ---

ensure_symlink "$CONFIGS/starship/starship.toml" "$HOME/.config/starship.toml"

# --- Vim ---

ensure_symlink "$CONFIGS/vim/init.lua" "$HOME/.config/nvim/init.lua"

# --- Zed ---

ensure_symlink "$CONFIGS/zed/settings.json" "$HOME/.config/zed/settings.json"
ensure_symlink "$CONFIGS/zed/keymap.json" "$HOME/.config/zed/keymap.json"

# --- Zellij ---

ensure_symlink "$CONFIGS/zellij/config.kdl" "$HOME/.config/zellij/config.kdl"
ensure_symlink "$CONFIGS/zellij/generate-cheatsheet.sh" "$HOME/.config/zellij/generate-cheatsheet.sh"

# --- Zsh ---

ensure_symlink "$CONFIGS/zsh/.zshrc" "$HOME/.zshrc"

log_info "Initialized ✨"
