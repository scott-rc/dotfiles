function gsearch --argument-names filter --description "Echos all commits matching the filter"
    set --function COMMITS (command git log --format='%h %s' --grep="$filter")

    if test -z "$COMMITS"
        echo "gsearch: No commits found matching filter $filter"
        return 1
    end

    for COMMIT in $COMMITS
        echo "$COMMIT"
    end
end
