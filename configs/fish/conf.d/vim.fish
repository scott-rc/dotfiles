set -gx EDITOR nvim
set -gx VISUAL nvim

if not status is-interactive
    return
end

alias vim=nvim

