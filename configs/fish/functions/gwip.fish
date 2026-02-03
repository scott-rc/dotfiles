function gwip --description "Commits all changes with the message WIP - <current time>"
    command git add --all
    command git commit --all --message "WIP - $(date +'%a, %b %d %I:%M %p')"
end
