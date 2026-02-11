#!/usr/bin/env bash
# Generates a keybinding cheatsheet from the Zellij config.
# Supports toggle (Cmd+/ again closes it) via lock file + PID.

set -euo pipefail

# --- Toggle mechanism ---
LOCKFILE="/tmp/zellij-cheatsheet-${ZELLIJ_SESSION_NAME:-default}.lock"

cleanup() { rm -f "$LOCKFILE"; }
trap cleanup EXIT

if [[ -f "$LOCKFILE" ]]; then
    old_pid=$(cat "$LOCKFILE" 2>/dev/null || true)
    if [[ -n "$old_pid" ]] && kill -0 "$old_pid" 2>/dev/null; then
        kill "$old_pid" 2>/dev/null || true
        rm -f "$LOCKFILE"
        exit 0
    fi
    rm -f "$LOCKFILE"
fi

echo $$ > "$LOCKFILE"

# --- Generate cheatsheet ---
CONFIG="$(dirname "$0")/config.kdl"

if [[ ! -f "$CONFIG" ]]; then
    echo "Error: config.kdl not found at $CONFIG" >&2
    exit 1
fi

awk '
BEGIN {
    mode = ""
    collecting = 0
    bind_key = ""
    action = ""
    depth = 0
    dim = "\033[2m"
    bold = "\033[1m"
    reset = "\033[0m"
}

# Skip comments
/^[[:space:]]*\/\// { next }

# Collecting multi-line bind body
collecting {
    for (i = 1; i <= length($0); i++) {
        c = substr($0, i, 1)
        if (c == "{") depth++
        if (c == "}") depth--
    }
    if (action == "") {
        a = $0
        gsub(/^[[:space:]]+/, "", a)
        gsub(/[[:space:]]*\{.*/, "", a)
        gsub(/;[[:space:]]*$/, "", a)
        if (a != "" && a != "}") action = a
    }
    if (depth <= 0) {
        if (action != "") {
            printf "%s%-26s%s %s%-20s%s %s\n", dim, mode, reset, bold, bind_key, reset, action
        }
        collecting = 0
        bind_key = ""
        action = ""
    }
    next
}

# Mode section headers
/^[[:space:]]+(locked|pane|tab|resize|move|scroll|search|session|entersearch|renametab|renamepane|tmux|shared_except|shared_among)[ {]/ {
    if (/bind/) { }
    else {
        line = $0
        gsub(/^[[:space:]]+/, "", line)
        gsub(/[[:space:]]*\{[[:space:]]*$/, "", line)
        if (line != mode) {
            mode = line
            display = mode
            gsub(/"/, "", display)
            if (display ~ /^shared_except /) {
                args = display
                sub(/^shared_except /, "", args)
                gsub(/ /, ", ", args)
                display = "shared (except " args ")"
            } else if (display ~ /^shared_among /) {
                args = display
                sub(/^shared_among /, "", args)
                gsub(/ /, ", ", args)
                display = "shared (among " args ")"
            }
        }
        next
    }
}

# Bind lines
/bind "/ {
    if (/^[[:space:]]*\/\//) next
    line = $0
    gsub(/^[[:space:]]+/, "", line)

    # Extract key (handle escaped quotes like \")
    sub(/^bind "/, "", line)
    if (substr(line, 1, 1) == "\\") {
        bind_key = substr(line, 1, 2)
        rest = substr(line, 4)
    } else {
        pos = index(line, "\"")
        if (pos == 0) next
        bind_key = substr(line, 1, pos - 1)
        rest = substr(line, pos + 1)
    }

    # Count braces
    opens = 0; closes = 0
    for (i = 1; i <= length($0); i++) {
        c = substr($0, i, 1)
        if (c == "{") opens++
        if (c == "}") closes++
    }

    if (opens == closes) {
        # Single-line bind
        sub(/^[[:space:]]*\{[[:space:]]*/, "", rest)
        sub(/[[:space:]]*\}[[:space:]]*$/, "", rest)
        gsub(/;[[:space:]]*$/, "", rest)
        if (rest != "" && rest !~ /^[[:space:]]*$/) {
            printf "%s%-26s%s %s%-20s%s %s\n", dim, mode, reset, bold, bind_key, reset, rest
        }
    } else {
        # Multi-line bind
        collecting = 1
        depth = opens - closes
        action = ""
        sub(/^[[:space:]]*\{[[:space:]]*/, "", rest)
        gsub(/;[[:space:]]*$/, "", rest)
        gsub(/[[:space:]]*\{.*/, "", rest)
        if (rest != "" && rest !~ /^[[:space:]]*$/) action = rest
    }
}
' "$CONFIG" | fzf --ansi --no-sort --layout=reverse \
    --header "ZELLIJ KEYBINDINGS  |  Type to filter  |  Esc to close" \
    --header-first --prompt "Filter: " --pointer="" \
    --no-multi --info=hidden --border=none \
    --highlight-line --bind "enter:abort" --no-scrollbar \
    || true
