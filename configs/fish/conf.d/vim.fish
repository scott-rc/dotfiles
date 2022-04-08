if not status is-interactive
    return
end

brew_ensure nvim neovim

function vim --wraps nvim
    command nvim $argv
end

abbr --add v nvim
