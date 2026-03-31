set -gx GIT_SPICE_NO_GS_WARNING 1

if not status is-interactive
    return
end

alias g=git

abbr --add ga git add
abbr --add gaa git add --all
abbr --add gb git branch
abbr --add gbD git branch -D
abbr --add gbd git branch --delete
abbr --add gbm git branch --move
abbr --add gc git commit --verbose
abbr --add gc! git commit --amend --no-edit
abbr --add gca git commit --all --verbose
abbr --add gca! git commit --all --amend --no-edit
abbr --add gcam 'git add --all && git commit --message'
abbr --add gcam! 'git add --all && git commit --verbose --amend'
abbr --add gcl git clone
abbr --add gcm git commit --message
abbr --add gcm! git commit --verbose --amend
abbr --add gco git checkout
abbr --add gco- git checkout -
abbr --add gcob git checkout -b
abbr --add gcoB git checkout -B
abbr --add gcom git checkout main
abbr --add gcp git cherry-pick
# abbr --add gd git diff
# abbr --add gds git diff --staged
# abbr --add gd~ git diff HEAD~
abbr --add gf git fetch
abbr --add gl git log
abbr --add gp git pull
abbr --add gps git push
abbr --add gr git reset
abbr --add grb git rebase
abbr --add grba git rebase --abort
abbr --add grbc git rebase --continue
abbr --add grbi git rebase -i
abbr --add grbm git rebase main
abbr --add grbmi git rebase main -i
abbr --add grh git reset --hard
abbr --add grh! 'git reset --hard && git clean -fd'
abbr --add grs git reset --soft
abbr --add gr~ git reset HEAD~
# abbr --add gs git status --short --branch
# abbr --add gsp git-spice
# abbr --add gst git stash
# abbr --add gstp git stash pop
# abbr --add gsts git stash save
abbr --add gs git-spice
abbr --add gsl git-spice ls
abbr --add gsrs git-spice repo sync
abbr --add gss git status --short --branch
abbr --add gssr git-spice stack restack
abbr --add gsss git-spice stack submit
