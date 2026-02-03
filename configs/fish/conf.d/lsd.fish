if not status is-interactive
    return
end

alias l=lsd
alias ls='lsd --almost-all --long --group-directories-first --blocks name'
alias la='lsd --almost-all --long --group-directories-first --blocks name,size,date,permission,user,group --header'
