# Completions for ksc (kubectl context switch)
complete -c ksc -f
complete -c ksc -l kubeconfig -d "Path to kubeconfig file" -r -F
complete -c ksc -a '(__fish_ksc_contexts)'

function __fish_ksc_contexts
    kubectl config get-contexts -o name 2>/dev/null
end
