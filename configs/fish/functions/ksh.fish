function ksh --description "Exec into a pod"
    argparse 'n/namespace=' 'c/context=' 'container=' 'shell=' -- $argv
    or return

    set -l kubectl_flags
    if set -q _flag_namespace
        set -a kubectl_flags -n $_flag_namespace
    end
    if set -q _flag_context
        set -a kubectl_flags --context $_flag_context
    end

    set -l container_flag
    if set -q _flag_container
        set container_flag -c $_flag_container
    end

    set -l pod
    if test (count $argv) -gt 0
        set pod $argv[1]
    else
        set pod (kubectl $kubectl_flags get pods -o name | sed 's|pod/||' | fzf --select-1)
        or return
    end

    if set -q _flag_shell
        kubectl $kubectl_flags exec -it $pod $container_flag -- $_flag_shell
    else
        kubectl $kubectl_flags exec -it $pod $container_flag -- sh -c "clear; (bash || ash || sh)"
    end
end
