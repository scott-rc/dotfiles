if not status is-interactive
    return
end

brew_ensure starship
starship init fish | source
