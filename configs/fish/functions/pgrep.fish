function pgrep --argument-names PATTERN --description "Search for processes matching pattern"
    for LINE in (ps -xo pid,command | rg $PATTERN)
        if string match --quiet --regex "^ *[0-9]+ rg $PATTERN\$" $LINE
            # skip the current process
            continue
        end

        set --append PIDS (echo $LINE | awk '{print $1}')
        set --append COMMANDS $LINE
    end

    if test -z "$COMMANDS"
        return 1
    end

    for COMMAND in $COMMANDS
        echo $COMMAND
    end
end
