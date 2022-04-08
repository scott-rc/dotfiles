function gwip --description "Commits all changes with the message WIP (amends previous commit if it contains WIP)"
    command git add --all

    switch "$(command git log -1 --pretty=%B)"
        case '*WIP*'
            command git commit --all --amend --no-edit
        case '*'
            command git commit --all --message WIP
    end
end

function gsquash --argument-names n --description "Squashes the last n commits"
    if test -z "$n" || test "$n" -lt 1
        echo "Argument n must be greater than 0"
        return 1
    end

    set --local n_plus_one (math $n + 1)
    set --local messages (command git log --abbrev HEAD~"$n_plus_one"..HEAD)

    echo "About to squash the following commits"
    for message in $messages
        echo "  $message"
    end
    echo ''

    read --function --prompt-str '(y/n) ' answer

    switch "$answer"
        case y yes
            # https://stackoverflow.com/a/5201642/5842886
            command git reset --soft "HEAD~$n"
            command git commit --edit --message (command git log --format=%B --reverse HEAD..HEAD@{"$n"})
    end
end

alias g=git

abbr --add ga git add
abbr --add gaa git add --all
abbr --add gb git branch
abbr --add gc git commit
abbr --add gc! git commit --amend --no-edit
abbr --add gca git commit --all
abbr --add gca! git commit --all --amend --no-edit
abbr --add gcam 'git add --all && git commit --message'
abbr --add gcam! 'git add --all && git commit --amend'
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
abbr --add gs git status --short --branch
abbr --add gst git stash
abbr --add gstp git stash pop
abbr --add gsts git stash save
