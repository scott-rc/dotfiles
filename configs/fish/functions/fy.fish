function fy --description "Fuzzy find file and yank contents to clipboard"
    set -l file (fzf_files --query "$argv")
    or return

    cat $file | pbcopy
    echo "Copied contents of: $file"
end
