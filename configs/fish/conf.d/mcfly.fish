if not status is-interactive
    return
end

brew_ensure mcfly

mcfly init fish | source
