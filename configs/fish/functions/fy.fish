function fy --description "Fuzzy find file and yank contents to clipboard"
    argparse 'u/unrestricted' -- $argv
    or return

    set -l unrestricted_flag
    if set -q _flag_unrestricted
        set unrestricted_flag -u
    end

    set -l file (fzf_files $unrestricted_flag --query "$argv")
    or return

    cat $file | pbcopy
    echo "Copied contents of: $file"
end
