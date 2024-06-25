if not status is-interactive
    return
end

brew_ensure delta git-delta

function gwip --description "Commits all changes with the message WIP - <current time>"
    command git add --all
    command git commit --all --message "WIP - $(TZ=America/Toronto date +'%a, %b %d %I:%M %p')"
end

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
                set --function SUBJECT "WIP - $(TZ=America/Toronto date +'%a, %b %d %I:%M %p')"
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


function gfixup
    open 'https://stackoverflow.com/a/27721031/5842886'
end

function gprune
    if not test (string match -r 'main|master' (git symbolic-ref --short HEAD))
        echo "gprune: must be on main branch to prune"
        return 1
    end

    # git branch --merged | grep -v "\*" | xargs -n 1 git branch -d
    git branch --format '%(refname:short) %(upstream:track)' | awk '$2 == "[gone]" { print $1 }' | xargs -r git branch -D
end

alias g=git
alias gl="git log --pretty=format:'%h %C(blue)%d%C(reset) %s' --graph --date=short --branches --decorate"

abbr --add ga git add
abbr --add gaa git add --all
abbr --add gb git branch
abbr --add gbm git branch --move
abbr --add gbd git branch --delete
abbr --add gbD git branch -D
abbr --add gc git commit --verbose
abbr --add gc! git commit --amend --no-edit
abbr --add gca git commit --all --verbose
abbr --add gca! git commit --all --amend --no-edit
abbr --add gcam 'git add --all && git commit --message'
abbr --add gcam! 'git add --all && git commit --verbose --amend'
abbr --add gcm git commit --message
abbr --add gcm! git commit --verbose --amend
abbr --add gcl git clone
abbr --add gco git checkout
abbr --add gco- git checkout -
abbr --add gcob git checkout -b
abbr --add gcom git checkout main
abbr --add gcp git cherry-pick
abbr --add gd git diff
abbr --add gds git diff --staged
abbr --add gd~ git diff HEAD~
abbr --add gf git fetch
abbr --add gp git pull
abbr --add gps git push
abbr --add gr git reset
abbr --add grh git reset --hard
abbr --add grh! 'git reset --hard && git clean -fd'
abbr --add grs git reset --soft
abbr --add gr~ git reset HEAD~
abbr --add grb git rebase
abbr --add grba git rebase --abort
abbr --add grbc git rebase --continue
abbr --add grbm git rebase main
abbr --add grbmi git rebase main -i
abbr --add gs git status --short --branch
abbr --add gst git stash
abbr --add gstp git stash pop
abbr --add gsts git stash save
