function fs --description "Fuzzy search file contents with ripgrep, view in bat"
    set -l selection (rg --line-number --color=always . | fzf --select-1 --query "$argv" --ansi --delimiter : --preview 'bat --color=always --highlight-line {2} {1}' --preview-window 'up,60%,+{2}-5')
    and begin
        set -l file (echo $selection | cut -d: -f1)
        set -l line (echo $selection | cut -d: -f2)
        bat --highlight-line $line $file
    end
end
