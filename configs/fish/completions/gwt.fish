complete -c gwt -f
complete -c gwt -s b -l branch -d 'Check out existing branch' -xa '(git branch --format="%(refname:short)" 2>/dev/null)'
complete -c gwt -s f -l from -d 'Base branch for new worktree' -xa '(git branch --format="%(refname:short)" 2>/dev/null)'
complete -c gwt -l force -d 'Force delete unmerged branch with same name'
complete -c gwt -s C -l repo -d 'Repository path' -ra '(__fish_complete_directories)'
