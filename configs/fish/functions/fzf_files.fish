function fzf_files --description "fzf with bat file preview"
    fzf --preview 'bat --color=always --style=numbers --line-range=:500 {}' --preview-window up,60% $argv
end
