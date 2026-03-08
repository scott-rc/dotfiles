#!/usr/bin/env bash

set -euo pipefail

RED="\033[31m"
GREEN="\033[32m"
YELLOW="\033[33m"
CYAN="\033[36m"
RESET="\033[0m"

LOG_LEVEL="${LOG_LEVEL:-info}"

_log_level_num() {
	case "$1" in
		debug) echo 0 ;;
		info) echo 1 ;;
		warn) echo 2 ;;
		error) echo 3 ;;
		*) echo 1 ;;
	esac
}

log_debug() {
	if [[ $(_log_level_num "$LOG_LEVEL") -le 0 ]]; then
		echo -e "${GREEN}[DEBUG]${RESET} $1"
	fi
}

log_info() {
	if [[ $(_log_level_num "$LOG_LEVEL") -le 1 ]]; then
		echo -e "${CYAN}[INFO]${RESET} $1"
	fi
}

log_warn() {
	if [[ $(_log_level_num "$LOG_LEVEL") -le 2 ]]; then
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

	local dir
	dir="$(dirname "$target")"

	log_info "Creating symlink from $target to $source"
	if [ -e "$target" ] || [ -L "$target" ]; then
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
