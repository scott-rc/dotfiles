# Completions for gw (git worktree switcher)
complete --command gw --no-files --arguments '(__fish_gw_worktrees)'

function __fish_gw_worktrees --description "List git worktree names for completion"
    git worktree list --porcelain 2>/dev/null | while read -l line
        if string match -q 'worktree *' $line
            set -l path (string replace 'worktree ' '' $line)
            basename $path
        end
    end
end
