if not status is-interactive
    return
end

brew_ensure nvim neovim

alias vim=nvim

abbr --add v nvim

function vf --description "Open file in nvim via fzf"
    set file (fzf_prompt vim $argv)
    and nvim $file
end
