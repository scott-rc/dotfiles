function gsw --description "Squashes consecutive WIP commits into one"
    set --function COMMITS (command git log --format='%s')
    set --function N 0

    for COMMIT in $COMMITS
        if echo "$COMMIT" | command grep -q -E '^WIP( - .+)?$'
            set --function N (math $N + 1)
        else
            break
        end
    end

    gsquash "$N"
end
