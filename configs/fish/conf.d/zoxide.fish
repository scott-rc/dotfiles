if not status is-interactive
    return
end

brew_ensure zoxide
zoxide init fish | source

function cd
    z $argv
end

function ci
    zi $argv
end
