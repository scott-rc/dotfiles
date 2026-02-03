if not status is-interactive
    return
end

direnv hook fish | source
