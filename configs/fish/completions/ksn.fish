# Completions for ksn (kubectl namespace switch)
complete -c ksn -f
complete -c ksn -l context -d "Kubernetes context" -r -a '(__fish_ksn_contexts)'
complete -c ksn -l kubeconfig -d "Path to kubeconfig file" -r -F
complete -c ksn -a '(__fish_ksn_namespaces)'

function __fish_ksn_contexts
    kubectl config get-contexts -o name 2>/dev/null
end

function __fish_ksn_namespaces
    set -l ctx_flag
    set -l cmdline (commandline -opc)
    if set -l idx (contains -i -- --context $cmdline)
        set ctx_flag --context $cmdline[(math $idx + 1)]
    end
    kubectl $ctx_flag get namespaces -o jsonpath='{.items[*].metadata.name}' 2>/dev/null | tr ' ' '\n'
end
