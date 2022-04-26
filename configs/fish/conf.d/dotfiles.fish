if not status is-interactive
    return
end

set --local updated_at_file "$DOTFILES/updated_at"
set --local now (date +%s)

if test (math "$now" - (cat "$updated_at_file" 2>/dev/null || echo 0)) -gt 3600
    echo 'dotfiles: updating...'
    if not git diff --quiet
        echo 'dotfiles: you have uncommited changes'
        return
    end

    pushd "$DOTFILES"
    git pull
    ./scripts/setup.ts
    echo "$now" >>"$updated_at_file"
    popd
end