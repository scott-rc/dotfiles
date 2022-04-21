if not status is-interactive
    return
end

brew_ensure zoxide
zoxide init fish | source

alias cd=z
alias ci=zi
