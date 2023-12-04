if not status is-interactive
    return
end

brew_ensure atuin

atuin init fish --disable-up-arrow | source
