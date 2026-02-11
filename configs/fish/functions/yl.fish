function yl --description "Fuzzy search file contents, copy path:line to clipboard"
    set -l selection (rg --line-number --color=always . | fzf --select-1 --query "$argv" --ansi --delimiter : --preview 'bat --color=always --highlight-line {2} {1}' --preview-window 'up,60%,+{2}-5')
    or return
    set -l file (echo $selection | cut -d: -f1)
    set -l line (echo $selection | cut -d: -f2)
    echo -n "$file:$line" | pbcopy
    echo "Copied: $file:$line"
end
