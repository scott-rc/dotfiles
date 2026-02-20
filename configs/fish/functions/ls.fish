function ls --wraps=lsd --description 'lsd with dimmed merged worktrees'
    set -l flags
    set -l positionals
    for arg in $argv
        switch $arg
            case '-*'
                set -a flags $arg
            case '*'
                set -a positionals $arg
        end
    end

    # Multiple positional args — passthrough without dimming
    if test (count $positionals) -gt 1
        command lsd --almost-all --long --group-directories-first --blocks name $argv
        return
    end

    set -l target (test (count $positionals) -ge 1; and echo $positionals[1]; or echo .)
    set -l resolved (realpath $target 2>/dev/null; or echo $target)

    if test (basename $resolved) != .worktrees
        command lsd --almost-all --long --group-directories-first --blocks name $argv
        return
    end

    # Find repo root (parent of .worktrees/)
    set -l repo_root (dirname $resolved)
    if not test -d $repo_root/.git
        command lsd --almost-all --long --group-directories-first --blocks name $argv
        return
    end

    set -l default_branch (git -C $repo_root rev-parse --abbrev-ref origin/HEAD 2>/dev/null | string replace 'origin/' '')
    if test -z "$default_branch"
        set default_branch main
    end

    # Collect worktree dir→branch mappings from porcelain output
    set -l wt_dirs
    set -l wt_branches
    set -l cur_dir
    set -l cur_branch
    for line in (git -C $repo_root worktree list --porcelain 2>/dev/null)
        if string match -qr '^worktree ' $line
            set cur_dir (string replace 'worktree ' '' $line)
            set cur_branch ''
        else if string match -qr '^branch ' $line
            set cur_branch (string replace -r '^branch refs/heads/' '' $line)
        else if test -z "$line" -a -n "$cur_dir" -a -n "$cur_branch"
            set -a wt_dirs $cur_dir
            set -a wt_branches $cur_branch
            set cur_dir ''
            set cur_branch ''
        end
    end
    # Handle last entry (porcelain may not end with blank line)
    if test -n "$cur_dir" -a -n "$cur_branch"
        set -a wt_dirs $cur_dir
        set -a wt_branches $cur_branch
    end

    # Detect orphaned directories (on disk but not in git worktree list)
    set -l known_basenames
    for d in $wt_dirs
        set -a known_basenames (basename $d)
    end
    set -l merged_dirs
    for entry in $resolved/*/
        set -l entry_name (basename $entry)
        if not contains $entry_name $known_basenames
            set -a merged_dirs $entry_name
        end
    end

    # Check each worktree branch for merge status
    for i in (seq (count $wt_dirs))
        set -l dir_name (basename $wt_dirs[$i])
        set -l branch $wt_branches[$i]

        # Regular merge: branch tip is ancestor of default branch
        if git -C $repo_root merge-base --is-ancestor refs/heads/$branch origin/$default_branch 2>/dev/null
            set -a merged_dirs $dir_name
            continue
        end

        # Squash-merge: upstream tracking ref was deleted
        set -l track (git -C $repo_root for-each-ref --format='%(upstream:track)' refs/heads/$branch 2>/dev/null)
        if test "$track" = '[gone]'
            set -a merged_dirs $dir_name
        end
    end

    if test (count $merged_dirs) -eq 0
        command lsd --almost-all --long --group-directories-first --blocks name --color=always $flags $target
        return
    end

    # Dim merged entries in lsd output
    command lsd --almost-all --long --group-directories-first --blocks name --color=always $flags $target | while read -l line
        set -l plain (string replace -ra '\e\[[0-9;]*m' '' $line)
        set -l dimmed no
        for d in $merged_dirs
            if test "$plain" = "$d"
                set dimmed yes
                break
            end
        end

        if test $dimmed = yes
            printf '\e[2m%s\e[0m\n' $line
        else
            echo $line
        end
    end
end
