function fzf_files --description "fzf with bat file preview"
    argparse 'u/unrestricted' -- $argv
    or return

    if set -q _flag_unrestricted
        eval $FZF_UNRESTRICTED_COMMAND | fzf --select-1 --preview 'bat --color=always --style=numbers --line-range=:500 {}' --preview-window up,60% $argv
    else
        fzf --select-1 --preview 'bat --color=always --style=numbers --line-range=:500 {}' --preview-window up,60% $argv
    end
end
