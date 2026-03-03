function v --description "Open in editor (no args: current directory)"
    if test (count $argv) -eq 0
        nvim .
        return
    end

    # Parse file:line, file:line:col, and file:line-endline patterns
    set -l args
    for arg in $argv
        if string match -rq '^(?<filepath>.+):(?<line>\d+)(?:[:-]\d+)?$' -- $arg
            set -a args "+$line" "$filepath"
        else
            set -a args "$arg"
        end
    end

    nvim $args
end
