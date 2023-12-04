if not status is-interactive
    return
end

function __dotfiles_dir
    echo (realpath (realpath (status dirname))/../../..)
end

function update_dotfiles
    set --local dotfiles_dir (__dotfiles_dir)
    pushd $dotfiles_dir

    if not git diff --quiet
        echo 'dotfiles: cannot update, you have uncommitted changes'
        popd
        return
    end

    git pull
    WORKSPACE_ROOT=$dotfiles_dir ./scripts/setup.ts
    popd
end

function edit_dotfiles
    code (__dotfiles_dir)
end
