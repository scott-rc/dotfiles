function fe --description "Fuzzy find file and open in nvim"
    set -l file (fzf_files -- --query "$argv")
    and nvim $file
end
