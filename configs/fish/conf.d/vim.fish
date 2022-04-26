if not status is-interactive
    return
end

brew_ensure nvim neovim

alias vim=nvim

abbr --add v nvim
