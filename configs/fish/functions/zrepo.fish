function zrepo
    set -l repos \
        ~/Code/personal/dotfiles \
        ~/Code/gadget/gadget \
        ~/Code/gadget/ggt \
        ~/Code/gadget/skipper \
        ~/Code/gadget/global-infrastructure

    set -l name
    set -l dir

    if test (count $argv) -gt 0
        # Direct attach: zrepo gadget
        set name $argv[1]
        for r in $repos
            if test (basename $r) = $name
                set dir $r
                break
            end
        end
        if test -z "$dir"
            echo "Unknown repo: $name"
            return 1
        end
    else
        # Fuzzy chooser
        set -l choice (for r in $repos; echo (basename $r)\t$r; end | fzf --with-nth=1 --delimiter='\t' | cut -f2)
        if test -z "$choice"
            return
        end
        set name (basename $choice)
        set dir $choice
    end

    # Set Ghostty tab title to repo name
    printf '\e]0;%s\a' $name

    zellij attach -c $name options --default-cwd $dir
end
