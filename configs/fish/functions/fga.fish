function fga --description "Fuzzy git add"
    set -l files (git status --short | fzf --select-1 --multi --query "$argv" | awk '{print $2}')
    test -n "$files" && git add $files
end
