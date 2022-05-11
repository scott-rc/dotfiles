if not status is-interactive
    return
end

brew_ensure delta git-delta

function gwip --description "Commits all changes with the message WIP <current time>"
    command git add --all
    command git commit --all --message "WIP - $(TZ=America/Toronto date +'%a, %b %d %I:%M %^p')"
end

function gsquash --argument-names N --description "Squashes the last N commits"
    if test -z "$N" -o "$N" -lt 2
        echo "gsquash: Argument N must be greater than 2"
        return 1
    end

    set --local logs (command git log --abbrev HEAD~"$N"..HEAD)

    echo "gsquash: About to squash the following commits"
    for log in $logs
        echo "  $log"
    end
    echo ''

    read --function --prompt-str '(y/n) ' answer

    switch "$answer"
        case y yes
            # https://stackoverflow.com/a/5201642/5842886
            set --local messages (command git log --format=%B HEAD~"$N"..HEAD)
            command git reset --soft "HEAD~$N"
            command git commit --edit --message "$messages"
    end
end

alias g=git

abbr --add ga git add
abbr --add gaa git add --all
abbr --add gb git branch
abbr --add gbm git branch --move
abbr --add gbd git branch --delete
abbr --add gc git commit --verbose
abbr --add gc! git commit --amend --no-edit
abbr --add gca git commit --all --verbose
abbr --add gca! git commit --all --amend --no-edit
abbr --add gcam 'git add --all && git commit --message'
abbr --add gcam! 'git add --all && git commit --verbose --amend'
abbr --add gcm git commit --message
abbr --add gcm! git commit --amend --message
abbr --add gcl git clone
abbr --add gco git checkout
abbr --add gcob git checkout -b
abbr --add gd git diff
abbr --add gds git diff --staged
abbr --add gd~ git diff HEAD~
abbr --add gf git fetch
abbr --add gl git log --pretty=oneline --abbrev-commit
abbr --add gp git pull
abbr --add gps git push
abbr --add gr git reset
abbr --add grh git reset --hard
abbr --add grs git reset --soft
abbr --add gr~ git reset HEAD~
abbr --add grb git rebase
abbr --add gs git status --short --branch
abbr --add gst git stash
abbr --add gstp git stash pop
abbr --add gsts git stash save
