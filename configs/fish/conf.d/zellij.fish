if not status is-interactive
    return
end

# Update Zellij tab name with git branch when changing directories or switching branches
# Uses --on-event fish_prompt which fires reliably after every command
function _zellij_update_tabname --on-event fish_prompt
    test -n "$ZELLIJ" || return

    set -l toplevel (git rev-parse --show-toplevel 2>/dev/null)
    set -l tab_name
    if test -n "$toplevel"
        # In a worktree, .git is a file - get base repo name from common dir
        if test -f "$toplevel/.git"
            set -l common_dir (git rev-parse --git-common-dir 2>/dev/null)
            # Validate common_dir before using it
            if test -n "$common_dir" -a -d "$common_dir"
                set tab_name (basename (dirname "$common_dir"))
            end
        else
            set tab_name (basename "$toplevel")
        end
    end

    # Fallback to PWD basename if tab_name is empty or invalid
    if test -z "$tab_name"
        set tab_name (basename "$PWD")
    end

    # Skip if name hasn't changed
    if test "$_zellij_last_tabname" = "$tab_name"
        return
    end
    set -g _zellij_last_tabname $tab_name
    zellij action rename-tab "$tab_name"
end

# Set pane name to "branch: command" when a command starts
function _zellij_update_panename_preexec --on-event fish_preexec
    test -n "$ZELLIJ" || return

    set -l branch (git symbolic-ref --short HEAD 2>/dev/null)
    test -n "$branch" || return

    set -l cmd $argv[1]
    set -l pane_name "$branch: $cmd"

    # Skip if name hasn't changed
    if test "$_zellij_panename" = "$pane_name"
        return
    end
    set -g _zellij_panename $pane_name
    set -g _zellij_cmd_running 1
    zellij action rename-pane "$pane_name"
end

# Set pane name to just "branch" when at prompt (no command running)
function _zellij_update_panename_prompt --on-event fish_prompt
    test -n "$ZELLIJ" || return

    # Only reset pane name if a command was running
    if not set -q _zellij_cmd_running
        return
    end
    set -e _zellij_cmd_running

    set -l branch (git symbolic-ref --short HEAD 2>/dev/null)
    if test -z "$branch"
        set -e _zellij_panename
        return
    end

    # Skip if already showing just the branch
    if test "$_zellij_panename" = "$branch"
        return
    end
    set -g _zellij_panename $branch
    zellij action rename-pane "$branch"
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
