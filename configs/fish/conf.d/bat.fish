if not status is-interactive
    return
end

brew_ensure bat

function cat --wraps bat
    command bat $argv
end
