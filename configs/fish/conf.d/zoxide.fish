if not status is-interactive
    return
end

zoxide init fish --cmd cd | source
