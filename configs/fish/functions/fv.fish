function fv --description "Fuzzy find file and view in bat"
    set -l file (fzf_files -- --query "$argv")
    and bat $file
end
