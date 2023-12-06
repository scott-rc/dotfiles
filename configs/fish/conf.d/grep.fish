if not status is-interactive
    return
end

brew_ensure rg ripgrep

alias grep=rg

function pgrep --argument-names PATTERN
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

function pkill --argument-names PATTERN
    set --function PIDS (pgrep $PATTERN | awk '{print $1}')

    echo "rgkill: Kill these processes?"
    echo ""
    for COMMAND in $COMMANDS
        echo $COMMAND
    end
    echo ""

    read --function --prompt-str '(y/n) ' ANSWER

    switch "$ANSWER"
        case y yes
            for PID in $PIDS
                kill -9 $PID
            end
    end
end
