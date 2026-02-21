function __gwt_active_basenames --description "List basenames of active worktrees for a repo"
    set -l repo $argv[1]
    for line in (command git -C $repo worktree list --porcelain 2>/dev/null)
        if string match -qr '^worktree ' $line
            basename (string replace 'worktree ' '' $line)
        end
    end
end
