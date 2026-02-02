# Completions for gw (git worktree switcher)
complete --command gw --no-files --arguments '(__fish_gw_worktrees)'

function __fish_gw_worktrees --description "List git worktree names for completion"
    for repo in ~/Code/*/*
        if test -d "$repo/.git"
            for wt in (git -C "$repo" worktree list --porcelain 2>/dev/null | grep '^worktree ' | sed 's/^worktree //')
                # Skip main worktree and Cursor worktrees
                if test "$wt" = "$repo"; or string match -q "$HOME/.cursor/worktrees/*" "$wt"
                    continue
                end
                basename $wt
            end
        end
    end
end
