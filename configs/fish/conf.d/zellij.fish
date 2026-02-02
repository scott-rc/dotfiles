if not status is-interactive
    return
end

brew_ensure zellij

# Update Zellij tab name with git branch when changing directories
# Uses --on-event fish_prompt which fires reliably after every command
function _zellij_update_tabname --on-event fish_prompt
    # Only run inside Zellij
    test -n "$ZELLIJ" || return

    # Skip if directory hasn't changed (avoid unnecessary zellij calls)
    test "$_zellij_last_pwd" = "$PWD" && return
    set -g _zellij_last_pwd $PWD

    # Get git branch, or use directory name if not in a git repo
    set -l branch (git symbolic-ref --short HEAD 2>/dev/null)
    if test -n "$branch"
        set -l toplevel (git rev-parse --show-toplevel 2>/dev/null)
        # In a worktree, .git is a file
        if test -f "$toplevel/.git"
            # Get base repo name from common git dir (e.g., /path/to/ggt/.git -> ggt)
            set -l common_dir (git rev-parse --git-common-dir 2>/dev/null)
            set -l base_repo (basename (dirname "$common_dir"))
            # Strip branch prefix (e.g., sc/fix-bug -> fix-bug)
            set -l short_branch (string replace -r '^[^/]+/' '' "$branch")
            zellij action rename-tab "$base_repo:$short_branch"
        else
            zellij action rename-tab "$(basename "$toplevel"):$branch"
        end
    else
        zellij action rename-tab (basename "$PWD")
    end
end

# Auto-attach to Zellij in IDE terminals (per-directory sessions)
if not is_truthy "$ZELLIJ"; and is_truthy "$ZELLIJ_AUTO_ATTACH"
    set -l session_name (basename $PWD)

    # Check if we're already inside this session (can happen when Cursor reuses terminals)
    set -l current_session (zellij list-sessions 2>/dev/null | string match "*current*" | string replace -ra "\e\[[0-9;]*m" "" | string split " " | head -1)
    if test "$current_session" = "$session_name"
        # Already in this session, skip attach
    else
        zellij attach -c "$session_name"
        set -l zellij_status $status

        # Only exit if Zellij exited normally (status 0) and auto-exit is enabled
        if test $zellij_status -eq 0; and is_truthy "$ZELLIJ_AUTO_EXIT"
            kill $fish_pid
        end
    end
end
