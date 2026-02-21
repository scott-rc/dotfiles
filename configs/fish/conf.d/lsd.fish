if not status is-interactive
    return
end

# GitHub Dark Default file type colors for lsd
set -gx LS_COLORS "di=38;2;88;166;255;1:ln=38;2;57;197;207:or=38;2;255;123;114:pi=38;2;210;153;34:so=38;2;188;140;255:bd=38;2;210;153;34;1:cd=38;2;210;153;34:ex=38;2;63;185;80:fi=0:no=0"

alias l=lsd
alias la='lsd --almost-all --long --group-directories-first --blocks name,size,date,permission,user,group --header'
