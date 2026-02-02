function fb --description "Fuzzy find file and view in bat"
    set -l file (fzf --query "$argv")
    and bat $file
end
