function fgl --description "Fuzzy git log browser"
    set -l commit (git log --oneline --color=always | fzf --ansi --query "$argv" --preview 'git show --color=always {1}' --preview-window up,60%)
    and git show (echo $commit | awk '{print $1}')
end
