# Completions for ksh (kubectl exec into pod)
complete -c ksh -e  # Erase system completions for Korn shell
complete -c ksh -f
complete -c ksh -s n -l namespace -d "Kubernetes namespace" -r -a '(__fish_ksh_namespaces)'
complete -c ksh -s c -l context -d "Kubernetes context" -r -a '(__fish_ksh_contexts)'
complete -c ksh -l container -d "Container name" -r -a ''
complete -c ksh -l shell -d "Shell to use" -r -a 'bash sh zsh'
complete -c ksh -a '(__fish_ksh_pods)'

function __fish_ksh_contexts
    kubectl config get-contexts -o name 2>/dev/null
end

function __fish_ksh_namespaces
    set -l flags (__kubectl_flags_from_cmdline context)
    kubectl $flags get namespaces -o jsonpath='{.items[*].metadata.name}' 2>/dev/null | tr ' ' '\n'
end

function __fish_ksh_pods
    set -l flags (__kubectl_flags_from_cmdline context namespace)
    kubectl $flags get pods -o json 2>/dev/null | jq -r '.items[] | select(.status.conditions[]? | select(.type=="Ready" and .status=="True")) | .metadata.name'
end
