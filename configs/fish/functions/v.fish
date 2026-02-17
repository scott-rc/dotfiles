function v --description "Open file/directory in editor, or fuzzy find"
    if test (count $argv) -gt 0 -a "$argv[1]" = -n
        nvim $argv[2..]
    else if test (count $argv) -gt 0 -a -e "$argv[1]"
        nvim $argv
    else
        vf $argv
    end
end
