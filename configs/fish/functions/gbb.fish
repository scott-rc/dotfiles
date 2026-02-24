function gbb --description "Print the base branch for the given or current branch"
    set -l target $argv[1]
    if test -z "$target"
        set target (command git branch --show-current 2>/dev/null)
    end

    set -l default_branch (command git rev-parse --abbrev-ref origin/HEAD 2>/dev/null | string replace 'origin/' '')
    test -z "$default_branch"; and set default_branch main

    if test -z "$target" -o "$target" = "$default_branch"
        echo $default_branch
        return
    end

    # Walk first-parent history looking for a commit decorated with another branch
    set -l skip "HEAD -> $target" "$target" "origin/$target"
    for line in (command git log --first-parent --format='%D' --decorate-refs=refs/heads/ --decorate-refs=refs/remotes/origin/ "$target" 2>/dev/null)
        test -z "$line"; and continue
        for ref in (string split ', ' $line)
            set -l name (string replace 'HEAD -> ' '' $ref)
            contains -- "$name" $skip; and continue
            # Skip stale branches whose tip is an ancestor of the default branch
            set -l candidate (string replace 'origin/' '' $name)
            if test "$candidate" != "$default_branch"
                command git merge-base --is-ancestor "$name" "$default_branch" 2>/dev/null; and continue
            end
            echo $candidate
            return
        end
    end

    echo $default_branch
end
