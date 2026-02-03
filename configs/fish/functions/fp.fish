function fp --description "Fuzzy find file and copy path to clipboard"
    argparse 'a/absolute' -- $argv
    or return

    set -l file (fzf_files --query "$argv")
    or return

    if set -q _flag_absolute
        set file (realpath $file)
    end

    echo -n $file | pbcopy
    echo "Copied: $file"
end
