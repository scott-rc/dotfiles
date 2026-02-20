function vd --description "Open Neovim diff viewer"
    set -l mode base
    set -l base ''

    if test (count $argv) -eq 0
        set mode base
    else if test "$argv[1]" = --staged
        set mode staged
    else if string match -q '*..*' "$argv[1]"
        # Range like main..feature — use left side as base
        set mode commit
        set base (string split '..' "$argv[1]")[1]
    else
        set mode commit
        set base "$argv[1]"
    end

    switch $mode
        case base
            nvim --cmd "let g:diff_viewer=1 | let g:diff_mode='base'"
        case staged
            nvim --cmd "let g:diff_viewer=1 | let g:diff_mode='staged'"
        case commit
            nvim --cmd "let g:diff_viewer=1 | let g:diff_mode='commit' | let g:diff_base='$base'"
    end
end
