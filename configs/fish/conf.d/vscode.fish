if not status is-interactive
    return
end

# function code --wraps code
#     if test (count $argv) -eq 0
#         echo 'code: Missing file/folder to open'
#         return 1
#     end

#     if test -f $argv[(count $argv)]
#         command code $argv
#         return
#     end

#     cd $argv[(count $argv)]

#     if test -f .envrc
#         direnv reload && command code .
#         return
#     end

#     command code .
# end

abbr --add c. code .
