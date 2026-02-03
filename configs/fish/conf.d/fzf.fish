if not status is-interactive
    return
end

set -l fd_excludes \
    .git \
    .DS_Store \
    .direnv

set -l fd_exclude_args (string join ' ' (for e in $fd_excludes; echo "--exclude $e"; end))
set -gx FZF_DEFAULT_COMMAND "fd --type f --hidden --follow --no-ignore $fd_exclude_args"
set -gx FZF_CTRL_T_COMMAND "$FZF_DEFAULT_COMMAND"
set -gx FZF_ALT_C_COMMAND "fd --type d --hidden --follow --no-ignore $fd_exclude_args"

set -gx FZF_DEFAULT_OPTS '--select-1 --height ~40% --reverse'
