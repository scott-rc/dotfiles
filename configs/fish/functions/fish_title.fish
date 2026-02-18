function fish_title
    set -l command $argv[1]
    if set -q command[1]; and test "$command" != fish
        echo -- (string sub -l 20 -- $command)
    else
        echo " "
    end
end
