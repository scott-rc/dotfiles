function fv --description "Fuzzy find file and open in nvim"
    set -l file (fzf --query "$argv")
    and nvim $file
end
