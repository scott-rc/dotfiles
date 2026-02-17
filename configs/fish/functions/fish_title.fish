function fish_title
    set -l command $argv[1]
    if not set -q command[1]; or test "$command" = fish
        set -l branch (git branch --show-current 2>/dev/null)
        if test -n "$branch"
            echo -- (string replace 'sc/' '' -- $branch)
        end
        return
    end
    echo -- (string sub -l 20 -- $command)
end
