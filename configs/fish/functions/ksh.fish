function ksh --argument-names POD --description "SSH into a pod"
    if test -z "$POD"
        echo "ksh: Argument POD is required"
        return 1
    end

    set --local POD_NAME (kubectl get pods -o custom-columns=':metadata.name' | grep "$POD" | gum choose --select-if-one)
    if test -z "$POD_NAME"
        echo "ksh: No pod selected"
        return 1
    end

    echo "ksh: SSHing into pod $POD_NAME"
    kubectl exec -it "$POD_NAME" -- bash
end
