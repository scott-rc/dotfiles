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

    set -l best ""
    set -l best_count 2147483647

    for branch in (command git for-each-ref --format='%(refname:short)' refs/heads/)
        test "$branch" = "$target"; and continue

        set -l mb (command git merge-base "$target" "$branch" 2>/dev/null)
        or continue

        set -l count (command git rev-list --count "$mb..$target" 2>/dev/null)
        or continue

        if test "$count" -lt "$best_count"
            set best_count $count
            set best $branch
        end
    end

    if test -n "$best"
        echo $best
    else
        echo $default_branch
    end
end
