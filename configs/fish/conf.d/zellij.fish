if not status is-interactive
    return
end

brew_ensure zellij

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
