function fish_title
    # An override for the current command is passed as the first parameter.
    set -l command $argv[1]
    if not set -q command[1]; or test "$command" = fish
        set command
    end
    echo -- (string sub -l 20 -- $command)
end
