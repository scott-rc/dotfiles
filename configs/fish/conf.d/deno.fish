if not status is-interactive
    return
end

brew_ensure deno

fish_add_path ~/.deno/bin
