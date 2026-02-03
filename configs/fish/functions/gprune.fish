function gprune --description "Prune branches with gone upstream"
    if not test (string match -r 'main|master' (git symbolic-ref --short HEAD))
        echo "gprune: must be on main branch to prune"
        return 1
    end

    git branch --format '%(refname:short) %(upstream:track)' | awk '$2 == "[gone]" { print $1 }' | xargs -r git branch -D
end
