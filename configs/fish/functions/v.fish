function v --description "Open in editor (no args: current directory)"
    if test (count $argv) -eq 0
        set -l last (nvim --headless -u NONE -c rshada \
            +'lua local cwd = vim.uv.cwd() .. "/"; for _, f in ipairs(vim.v.oldfiles) do if f:sub(1, #cwd) == cwd and vim.uv.fs_stat(f) then io.write(f); break end end' \
            +qa 2>/dev/null)
        if test -n "$last"
            nvim '+normal! g`"' "$last"
        else
            nvim .
        end
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
