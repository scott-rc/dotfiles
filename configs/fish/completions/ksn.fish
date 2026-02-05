# Completions for ksn (kubectl namespace switch)
complete -c ksn -f
complete -c ksn -s c -l context -d "Kubernetes context" -r -a '(__fish_ksn_contexts)'
complete -c ksn -l kubeconfig -d "Path to kubeconfig file" -r -F
complete -c ksn -a '(__fish_ksn_namespaces)'

function __fish_ksn_contexts
    set -l flags (__kubectl_flags_from_cmdline kubeconfig)
    kubectl $flags config get-contexts -o name 2>/dev/null
end

function __fish_ksn_namespaces
    set -l flags (__kubectl_flags_from_cmdline context kubeconfig)
    kubectl $flags get namespaces -o jsonpath='{.items[*].metadata.name}' 2>/dev/null | tr ' ' '\n'
end
