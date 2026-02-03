if not status is-interactive
    return
end

set -gx FZF_DEFAULT_COMMAND 'fd --type f --hidden --follow --exclude .git'
set -gx FZF_CTRL_T_COMMAND "$FZF_DEFAULT_COMMAND"
set -gx FZF_ALT_C_COMMAND 'fd --type d --hidden --follow --exclude .git'

# Use bat for fzf preview by default
set -gx FZF_DEFAULT_OPTS '--preview "bat --color=always --style=numbers --line-range=:500 {}" --preview-window up --select-1 --height ~40% --reverse'
