if not status is-interactive
    return
end

brew_ensure zoxide
zoxide init fish | source

function cd --argument-names DIR --wraps z
    if test -z "$DIR"
        z
        return
    end

    if test "$DIR" = -
        z -
        return
    end

    if test -d (realpath "$DIR")
        z $argv
        return
    end

    zi $argv
end

alias ci=zi
