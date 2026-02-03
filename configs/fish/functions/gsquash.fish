function gsquash --argument-names N SUBJECT --description "Squashes the last N commits"
    if test -z "$N" -o "$N" -lt 2
        echo "gsquash: Argument N must be greater than 2"
        return 1
    end

    set --function COMMITS (command git log --format='%h %s' HEAD~"$N"..HEAD)

    echo "gsquash: Squash the following commits?"
    echo ""
    for COMMIT in $COMMITS
        echo "$COMMIT"
    end
    echo ""

    read --function --prompt-str '(y/n) ' ANSWER

    switch "$ANSWER"
        case y yes
            if test -z "$SUBJECT"
                set --function SUBJECT "WIP - $(date +'%a, %b %d %I:%M %p')"
            end

            set --function ARGS
            for LINE in (command git log --format=%s HEAD~"$N"..HEAD)
                set --function --append ARGS --message
                set --function --append ARGS "- $LINE"
            end

            command git reset --soft "HEAD~$N"
            command git commit --edit --message "$SUBJECT" $ARGS
    end
end
