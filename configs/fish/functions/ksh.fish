function ksh --description "Exec into a pod"
    argparse 'n/namespace=' 'context=' 'c/container=' 'shell=' -- $argv
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

    set -l shell_cmd (test -n "$_flag_shell" && echo $_flag_shell || echo sh)

    set -l pod (kubectl $kubectl_flags get pods -o name | sed 's|pod/||' | fzf --select-1 --query "$argv")
    and kubectl $kubectl_flags exec -it $pod $container_flag -- $shell_cmd
end
