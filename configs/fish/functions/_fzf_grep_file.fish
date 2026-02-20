function _fzf_grep_file --description "Ctrl+G: rg content search picker"
    set -l selection (rg --line-number --color=always . | fzf --select-1 --query (commandline -t) --ansi --delimiter : \
        --preview 'bat --color=always --highlight-line {2} {1}' \
        --preview-window 'up,60%,+{2}-5')
    or begin
        commandline -f repaint
        return
    end
    set -l file (string split -m2 : -- $selection)[1]
    set -l line (string split -m2 : -- $selection)[2]
    commandline -t -- (string escape -- "$file:$line")
    commandline -f repaint
end
