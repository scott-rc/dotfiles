if not status is-interactive
    return
end

brew_ensure zoxide
zoxide init fish | source

function cd --wraps z
    if test -z $argv
        return (z)
    end

    if test -d $argv
        return (z $argv)
    end

    return (zi $argv)
end

alias ci=zi
