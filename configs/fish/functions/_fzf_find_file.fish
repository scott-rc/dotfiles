function _fzf_find_file --description "Ctrl+F: fzf file picker"
    set -l result (fzf_files --query (commandline -t))
    or begin
        commandline -f repaint
        return
    end
    commandline -t -- (string escape -- $result)
    commandline -f repaint
    # Auto-execute if there's already a command on the line (e.g. `v <ctrl+f>`)
    if test (count (commandline -o)) -gt 1
        commandline -f execute
    end
end
