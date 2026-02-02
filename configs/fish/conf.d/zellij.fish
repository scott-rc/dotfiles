if not status is-interactive
    return
end

brew_ensure zellij

# Auto-attach to Zellij in VSCode or Zed terminals (per-directory sessions)
if not is_truthy "$ZELLIJ"; and is_truthy "$ZELLIJ_AUTO_ATTACH"
    zellij attach -c (basename $PWD)

    if is_truthy "$ZELLIJ_AUTO_EXIT"
        kill $fish_pid
    end
end
