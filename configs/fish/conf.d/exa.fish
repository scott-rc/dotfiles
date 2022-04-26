if not status is-interactive
    return
end

brew_ensure exa

alias l=exa
alias ls=exa

abbr --add la exa --all --long --header --git
abbr --add lg exa --all --long --header --git --grid
