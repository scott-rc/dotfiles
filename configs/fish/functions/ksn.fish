function ksn --description "Switch kubectl namespace"
    argparse 'context=' 'kubeconfig=' -- $argv
    or return

    set -l kubectl_flags
    if set -q _flag_context
        set -a kubectl_flags --context $_flag_context
    end
    if set -q _flag_kubeconfig
        set -a kubectl_flags --kubeconfig $_flag_kubeconfig
    end

    set -l ns (kubectl $kubectl_flags get namespaces -o jsonpath='{.items[*].metadata.name}' | tr ' ' '\n' | fzf --select-1 --query "$argv")
    and kubectl $kubectl_flags config set-context --current --namespace=$ns
end
