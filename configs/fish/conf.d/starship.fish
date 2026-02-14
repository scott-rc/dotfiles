if not status is-interactive
    return
end

function _starship_dir --on-variable PWD
    # Check if in a git worktree at root where dir matches branch suffix
    set -l git_dir (git rev-parse --git-dir 2>/dev/null)
    if test $status -eq 0
        set -l common_dir (git rev-parse --git-common-dir 2>/dev/null)
        if test "$git_dir" != "$common_dir"
            # In a worktree
            set -l toplevel (git rev-parse --show-toplevel 2>/dev/null)
            if test "$PWD" = "$toplevel"
                # At worktree root — check if dir name matches branch suffix
                set -l branch (git symbolic-ref --short HEAD 2>/dev/null)
                if test $status -eq 0
                    set -l branch_suffix (string replace -r '^[^/]+/' '' -- $branch)
                    if test (basename $PWD) = "$branch_suffix"
                        set -e STARSHIP_DIR
                        return
                    end
                end
            end
        end
    end

    # Default: set directory for starship to display
    set -l toplevel (git rev-parse --show-toplevel 2>/dev/null)
    if test $status -eq 0
        if test "$PWD" = "$toplevel"
            set -gx STARSHIP_DIR (basename $toplevel)
        else
            set -l rel (string replace "$toplevel/" '' -- $PWD)
            set -gx STARSHIP_DIR (basename $toplevel)/$rel
        end
    else
        set -gx STARSHIP_DIR (prompt_pwd)
    end
end

_starship_dir # initialize on shell start

starship init fish | source
