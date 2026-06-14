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

log_section() {
	local title="$1"
	if [[ $(_log_level_num "$LOG_LEVEL") -le 1 ]]; then
		echo ""
		echo -e "${CYAN}--- $title ---${RESET}"
	fi
}

log_success() {
	if [[ $(_log_level_num "$LOG_LEVEL") -le 1 ]]; then
		echo -e "  ${GREEN}✔${RESET} $1"
	fi
}

# Run a command with a spinner animation.
# Usage: run_with_spinner "label" command [args...]
# Behavior by mode (so output stays useful both live and in captured logs):
# - debug (LOG_LEVEL=debug): no spinner, command output streams live
# - quiet (LOG_LEVEL=warn/error): runs silently
# - interactive TTY: animated spinner; on failure, dumps captured output
# - non-TTY (piped/redirected to a file): no animation (no \r spam); prints a
#   plain "▸ label" line, then "✔/✖ label"; on failure, dumps captured output
# On failure it always returns the command's non-zero exit code.
run_with_spinner() {
	local label="$1"
	shift

	local level
	level=$(_log_level_num "$LOG_LEVEL")

	# Silent mode (warn/error): run without any output
	if [[ $level -gt 1 ]]; then
		"$@" >/dev/null 2>&1
		return
	fi

	# Debug mode: skip spinner, stream output live for easy debugging
	if [[ $level -le 0 ]]; then
		echo -e "  ${CYAN}▸${RESET} $label"
		local exit_code=0
		"$@" || exit_code=$?
		if [[ $exit_code -eq 0 ]]; then
			echo -e "  ${GREEN}✔${RESET} $label"
		else
			echo -e "  ${RED}✖${RESET} $label ${RED}(exit $exit_code)${RESET}"
		fi
		return "$exit_code"
	fi

	local logfile exit_code=0
	logfile=$(mktemp)

	if [[ -t 1 ]]; then
		# Interactive TTY: animate a spinner in a subshell while the command runs.
		(
			local spinner_chars='⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏'
			local pid

			"$@" > "$logfile" 2>&1 &
			pid=$!

			trap 'kill $pid 2>/dev/null' EXIT

			local i=0
			while kill -0 "$pid" 2>/dev/null; do
				local char="${spinner_chars:i%${#spinner_chars}:1}"
				printf "\r  %s %s" "$char" "$label"
				i=$((i + 1))
				sleep 0.08
			done

			trap - EXIT

			local code=0
			wait "$pid" || code=$?

			printf "\r\033[K"

			exit "$code"
		)
		exit_code=$?
	else
		# Non-interactive (piped/redirected): no animation, just capture output.
		echo -e "  ${CYAN}▸${RESET} $label"
		"$@" > "$logfile" 2>&1 || exit_code=$?
	fi

	if [[ $exit_code -eq 0 ]]; then
		echo -e "  ${GREEN}✔${RESET} $label"
	else
		echo -e "  ${RED}✖${RESET} $label"
		echo -e "  ${RED}Command failed with exit code $exit_code. Output:${RESET}"
		sed 's/^/    /' "$logfile"
		echo -e "  ${YELLOW}Re-run with LOG_LEVEL=debug for live output.${RESET}"
	fi

	rm -f "$logfile"

	return "$exit_code"
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
