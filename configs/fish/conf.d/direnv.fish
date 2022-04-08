if not status is-interactive
    return
end

brew_ensure direnv

direnv hook fish | source
