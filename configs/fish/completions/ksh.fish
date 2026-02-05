# Completions for ksh (kubectl exec into pod)
complete -c ksh -f
complete -c ksh -s n -l namespace -d "Kubernetes namespace" -r -a '(__fish_ksh_namespaces)'
complete -c ksh -l context -d "Kubernetes context" -r -a '(__fish_ksh_contexts)'
complete -c ksh -s c -l container -d "Container name" -r
complete -c ksh -l shell -d "Shell to use" -r -a 'bash sh zsh'
complete -c ksh -a '(__fish_ksh_pods)'

function __fish_ksh_contexts
    kubectl config get-contexts -o name 2>/dev/null
end

function __fish_ksh_namespaces
    set -l ctx_flag
    set -l cmdline (commandline -opc)
    if set -l idx (contains -i -- --context $cmdline)
        set ctx_flag --context $cmdline[(math $idx + 1)]
    end
    kubectl $ctx_flag get namespaces -o jsonpath='{.items[*].metadata.name}' 2>/dev/null | tr ' ' '\n'
end

function __fish_ksh_pods
    set -l kubectl_flags
    set -l cmdline (commandline -opc)
    if set -l idx (contains -i -- -n $cmdline)
        set -a kubectl_flags -n $cmdline[(math $idx + 1)]
    else if set -l idx (contains -i -- --namespace $cmdline)
        set -a kubectl_flags -n $cmdline[(math $idx + 1)]
    end
    if set -l idx (contains -i -- --context $cmdline)
        set -a kubectl_flags --context $cmdline[(math $idx + 1)]
    end
    kubectl $kubectl_flags get pods -o name 2>/dev/null | sed 's|pod/||'
end
