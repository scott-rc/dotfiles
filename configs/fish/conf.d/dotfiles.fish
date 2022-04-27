if not status is-interactive
    return
end

function __dotfiles_path
    echo (realpath (realpath (status dirname))/../../..)
end

function update_dotfiles
    echo 'dotfiles: updating...'
    pushd (__dotfiles_path)

    if not git diff --quiet
        echo 'dotfiles: cannot update (you have uncommited changes)'
        popd
        return
    end

    git pull
    ./scripts/setup.ts
    echo (date +%s) >./updated_at
    popd
end

function edit_dotfiles
    code (__dotfiles_path)
end

set --local updated_at (cat (__dotfiles_path)"/updated_at" 2>/dev/null || echo 0)

if test (math (date +%s) - "$updated_at") -gt 3600
    update_dotfiles
end
