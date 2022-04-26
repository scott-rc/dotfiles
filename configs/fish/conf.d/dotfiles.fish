if not status is-interactive
    return
end

set --local updated_at_file "$DOTFILES/updated_at"
set --local now (date +%s)

if test (math "$now" - (cat "$updated_at_file" 2>/dev/null || echo 0)) -gt 3600
    echo 'dotfiles: updating...'
    pushd "$DOTFILES"

    if not git diff --quiet
        echo 'dotfiles: cannot update (you have uncommited changes)'
        popd
        return
    end

    git pull
    ./scripts/setup.ts
    echo "$now" >"$updated_at_file"
    popd
end

function edit_dotfiles
    code "$DOTFILES"
end
