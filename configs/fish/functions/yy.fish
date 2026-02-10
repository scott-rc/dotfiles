function yy --description "Yank file contents to clipboard"
    argparse 'u/unrestricted' -- $argv
    or return

    set -l unrestricted_flag
    if set -q _flag_unrestricted
        set unrestricted_flag -u
    end

    set -l file
    if test (count $argv) -ge 1 -a -f "$argv"
        set file $argv
    else
        set file (fzf_files $unrestricted_flag --query "$argv")
        or return
    end

    cat $file | pbcopy
    echo "Copied contents of: $file"
end
