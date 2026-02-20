complete -c vd -f
complete -c vd -l staged -d 'Show staged changes'
complete -c vd -a '(__fish_git_branches)'
complete -c vd -a '(git log --oneline -15 2>/dev/null | string split -f1 " ")'
