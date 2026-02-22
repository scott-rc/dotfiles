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

    # Collect candidate branches: local + remote tracking (deduplicated)
    set -l seen
    set -l candidates

    for branch in (command git for-each-ref --format='%(refname:short)' refs/heads/)
        test "$branch" = "$target"; and continue
        set -a seen $branch
        set -a candidates $branch
    end

    for ref in (command git for-each-ref --format='%(refname:short)' refs/remotes/origin/)
        set -l name (string replace 'origin/' '' $ref)
        test "$name" = "$target"; and continue
        test "$name" = HEAD; and continue
        contains -- $name $seen; and continue
        set -a candidates $ref
    end

    # Two-pass: first prefer ancestors (candidate_ahead == 0), then fall back to closest
    set -l best ""
    set -l best_count 2147483647
    set -l fallback ""
    set -l fallback_count 2147483647

    for branch in $candidates
        set -l mb (command git merge-base "$target" "$branch" 2>/dev/null)
        or continue

        set -l ahead (command git rev-list --count "$mb..$target" 2>/dev/null)
        or continue

        set -l candidate_ahead (command git rev-list --count "$mb..$branch" 2>/dev/null)
        or continue

        if test "$candidate_ahead" -eq 0
            # True ancestor: target was branched from this
            if test "$ahead" -lt "$best_count"
                set best_count $ahead
                set best $branch
            end
        else
            # Sibling/diverged: track as fallback
            if test "$ahead" -lt "$fallback_count"
                set fallback_count $ahead
                set fallback $branch
            end
        end
    end

    set -l result ""
    if test -n "$best"
        set result $best
    else if test -n "$fallback"
        set result $fallback
    else
        echo $default_branch
        return
    end

    # Strip origin/ prefix for display
    echo (string replace 'origin/' '' $result)
end
