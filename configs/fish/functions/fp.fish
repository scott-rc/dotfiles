function fp --description "Fuzzy find file and copy path to clipboard"
    argparse 'a/absolute' 'u/unrestricted' -- $argv
    or return

    set -l unrestricted_flag
    if set -q _flag_unrestricted
        set unrestricted_flag -u
    end

    set -l file (fzf_files $unrestricted_flag --query "$argv")
    or return

    if set -q _flag_absolute
        set file (realpath $file)
    end

    echo -n $file | pbcopy
    echo "Copied: $file"
end
