function zrepo
    set -l name
    set -l dir

    if test (count $argv) -gt 0
        set -l target $argv[1]
        set -l matches
        for repo in ~/Code/*/*
            test -d "$repo/.git"; or continue
            set -l rel (string replace -r '^.*/Code/' '' -- $repo)
            if test "$rel" = "$target"; or test (basename $repo) = "$target"
                set -a matches $repo
            end
        end
        if test (count $matches) -eq 0
            echo "Unknown repo: $target"
            return 1
        else if test (count $matches) -gt 1
            echo "Ambiguous repo '$target', matches:"
            for m in $matches
                echo "  "(string replace -r '^.*/Code/' '' -- $m)
            end
            return 1
        end
        set dir $matches[1]
        set name (basename $dir)
    else
        set -l choice (
            for repo in ~/Code/*/*
                test -d "$repo/.git"; or continue
                set -l rel (string replace -r '^.*/Code/' '' -- $repo)
                echo $rel\t$repo
            end | sort | fzf \
                --with-nth=1 \
                --delimiter='\t' \
                --preview 'echo "  "{2}; echo; echo "  branch: "$(git -C {2} branch --show-current 2>/dev/null); echo; echo "  recent commits:"; git -C {2} log --oneline -5 --color=always 2>/dev/null | sed "s/^/    /"; echo; echo "  status:"; git -C {2} status --short 2>/dev/null | head -10 | sed "s/^/    /"' \
                --preview-window=right:50% \
            | cut -f2
        )
        if test -z "$choice"
            return
        end
        set name (basename $choice)
        set dir $choice
    end

    printf '\e]0;%s\a' $name
    zellij attach -c $name options --default-cwd $dir
end
