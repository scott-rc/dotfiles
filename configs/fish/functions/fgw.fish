function fgw --argument-names query --description "Switch to a git worktree"
    # Collect worktrees from all repos in ~/Code/*/*
    # Excludes main worktrees and ~/.cursor/worktrees/*
    set -l worktrees
    for repo in ~/Code/*/*
        if test -d "$repo/.git"
            for wt in (git -C "$repo" worktree list --porcelain | grep '^worktree ' | sed 's/^worktree //')
                if test "$wt" = "$repo"; or string match -q "$HOME/.cursor/worktrees/*" "$wt"
                    continue
                end
                set -a worktrees $wt
            end
        end
    end

    if test (count $worktrees) -eq 0
        echo "No worktrees found"
        return 1
    end

    # Build display list - parse [group] from worktree name (format: <repo>-<branch>)
    set -l display_items
    for wt in $worktrees
        set -l name (basename $wt)
        set -l group (string split -m1 '-' $name)[1]
        set -l branch (string replace "$group-" '' $name)
        set -a display_items "[$group] $branch"
    end

    set -l selected_display (printf '%s\n' $display_items | fzf_prompt "Worktree" "$query")
    if test -z "$selected_display"
        return 0 # User cancelled
    end

    # Extract path from display format: "[group] branch" -> find matching worktree
    set -l selected_group (string match -r '^\[([^\]]+)\]' $selected_display)[2]
    set -l selected_branch (string replace -r '^\[.*\] ' '' $selected_display)
    set -l selected_name "$selected_group-$selected_branch"
    set -l selected
    for wt in $worktrees
        if test (basename $wt) = "$selected_name"
            set selected $wt
            break
        end
    end

    if test -z "$selected"
        echo "Error: Could not find worktree path"
        return 1
    end

    cd "$selected"
end
