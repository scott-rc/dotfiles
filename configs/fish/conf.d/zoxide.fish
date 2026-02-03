if not status is-interactive
    return
end

zoxide init fish | source

alias cd=z
alias ci=zi
