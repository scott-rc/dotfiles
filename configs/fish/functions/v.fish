function v --description "Open in editor (no args: current directory)"
    if test (count $argv) -eq 0
        nvim .
    else
        nvim $argv
    end
end
