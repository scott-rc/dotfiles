if not status is-interactive
    return
end

brew_ensure fzf

set -gx FZF_DEFAULT_COMMAND 'fd --type f --hidden --follow --exclude .git'
set -gx FZF_CTRL_T_COMMAND "$FZF_DEFAULT_COMMAND"
set -gx FZF_ALT_C_COMMAND 'fd --type d --hidden --follow --exclude .git'

# Use bat for fzf preview by default
set -gx FZF_DEFAULT_OPTS '--preview "bat --color=always --style=numbers --line-range=:500 {}" --select-1 --height ~40% --reverse'

function fzf_prompt --argument-names PROMPT QUERY
    if test -z "$PROMPT"
        set PROMPT ""
    end

    if test -z "$QUERY"
        set QUERY ""
    end

    fzf --prompt "$PROMPT: " --query "$QUERY" --select-1 --height ~40% --reverse
end
