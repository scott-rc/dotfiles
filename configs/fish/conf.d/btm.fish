if not status is-interactive
    return
end

brew_ensure btm bottom

function top --wraps btm
    command btm $argv
end
