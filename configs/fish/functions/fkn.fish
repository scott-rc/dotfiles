function fkn --description "Fuzzy kubectl namespace switch"
    set -l ns (kubectl get namespaces -o jsonpath='{.items[*].metadata.name}' | tr ' ' '\n' | fzf --select-1 --query "$argv")
    and kubectl config set-context --current --namespace=$ns
end
