if not status is-interactive
    return
end

brew_ensure zellij

# Auto-attach to Zellij in VSCode or Zed terminals (per-directory sessions)
if not set -q ZELLIJ; and set -q ZELLIJ_AUTO_ATTACH
    zellij attach -c (basename $PWD)

    if test "$ZELLIJ_AUTO_EXIT" = true
        kill $fish_pid
    end
end
