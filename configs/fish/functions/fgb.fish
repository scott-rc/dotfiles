function fgb --description "Fuzzy git branch checkout"
    set -l branch (git branch --all | grep -v HEAD | sed 's/^[* ]*//' | sed 's|remotes/origin/||' | sort -u | fzf --select-1 --query "$argv")
    or return
    git checkout $branch
end
