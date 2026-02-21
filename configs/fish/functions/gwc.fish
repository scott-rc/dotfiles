function gwc --description "Clean up merged and orphaned worktrees"
    # Resolve .worktrees directory
    set -l wt_root $argv[1]
    if test -z "$wt_root"
        set -l repo (command git rev-parse --show-toplevel 2>/dev/null)
        or begin
            echo "gwc: not in a git repo"
            return 1
        end
        set wt_root $repo/.worktrees
    end
    set wt_root (realpath $wt_root 2>/dev/null; or echo $wt_root)

    # Guards
    if not test -d $wt_root
        echo "gwc: $wt_root does not exist"
        return 1
    end
    set -l repo_root (dirname $wt_root)
    if not test -d $repo_root/.git
        echo "gwc: $repo_root is not a git repo"
        return 1
    end

    # Default branch
    set -l default_branch (command git -C $repo_root rev-parse --abbrev-ref origin/HEAD 2>/dev/null | string replace 'origin/' '')
    if test -z "$default_branch"
        set default_branch main
    end

    # Parse porcelain output for dir→branch mappings
    set -l wt_dirs
    set -l wt_branches
    set -l cur_dir
    set -l cur_branch
    for line in (command git -C $repo_root worktree list --porcelain 2>/dev/null)
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

    # Build set of known basenames from git worktree list
    set -l known_basenames (__gwt_active_basenames $repo_root)

    set -l stale_dirs
    set -l stale_labels
    set -l stale_branches

    # Detect orphaned directories (on disk but not in git worktree list)
    for entry in $wt_root/*/
        set -l entry_name (basename $entry)
        if not contains $entry_name $known_basenames
            set -a stale_dirs $entry_name
            set -a stale_labels orphaned
            set -a stale_branches ''
        end
    end

    # Detect merged/gone branches among tracked worktrees
    for i in (seq (count $wt_dirs))
        set -l dir_name (basename $wt_dirs[$i])
        set -l branch $wt_branches[$i]

        # Never clean the default branch
        if test "$branch" = "$default_branch"
            continue
        end

        # Regular merge: branch tip is ancestor of default branch
        if command git -C $repo_root merge-base --is-ancestor refs/heads/$branch origin/$default_branch 2>/dev/null
            set -a stale_dirs $dir_name
            set -a stale_labels merged
            set -a stale_branches $branch
            continue
        end

        # Squash-merge: upstream tracking ref was deleted
        set -l track (command git -C $repo_root for-each-ref --format='%(upstream:track)' refs/heads/$branch 2>/dev/null)
        if test "$track" = '[gone]'
            set -a stale_dirs $dir_name
            set -a stale_labels gone
            set -a stale_branches $branch
        end
    end

    # Nothing to clean
    if test (count $stale_dirs) -eq 0
        echo "gwc: no stale worktrees found"
        return 0
    end

    # Show what will be deleted
    echo "gwc: Remove the following worktrees?"
    echo ""
    for i in (seq (count $stale_dirs))
        printf "  %s (%s)\n" $stale_dirs[$i] $stale_labels[$i]
    end
    echo ""
    read -l --prompt-str '(y/n) ' answer

    function _gwc_rm
        chmod -R u+w $argv[1] 2>/dev/null
        rm -rf $argv[1]
    end

    switch "$answer"
        case y yes
            for i in (seq (count $stale_dirs))
                set -l dir $stale_dirs[$i]
                set -l full_path $wt_root/$dir
                if test "$stale_labels[$i]" = orphaned
                    _gwc_rm $full_path
                else
                    command git -C $repo_root worktree remove --force $full_path 2>/dev/null
                    or _gwc_rm $full_path
                    # Delete the branch if it still exists
                    command git -C $repo_root branch -D $stale_branches[$i] 2>/dev/null
                end
                echo "  removed $dir"
            end
    end
end
