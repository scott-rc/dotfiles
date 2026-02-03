function fkp --description "Fuzzy kubectl pod exec"
    set -l pod (kubectl get pods -o name | sed 's|pod/||' | fzf --select-1 --query "$argv")
    and kubectl exec -it $pod -- sh
end
