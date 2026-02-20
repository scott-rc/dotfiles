function _fzf_find_file --description "Ctrl+F: fzf file picker"
    set -l result (fzf_files --query (commandline -t))
    and commandline -t -- (string escape -- $result)
    commandline -f repaint
end
