complete -c vd -f
complete -c vd -l staged -d 'Show staged changes'
complete -c vd -a '(git branch --format="%(refname:short)" 2>/dev/null)'
complete -c vd -a '(git log --oneline -15 2>/dev/null | string split -f1 " ")'
