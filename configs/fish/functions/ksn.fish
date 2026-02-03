function ksn --argument-names NAMESPACE --description "Switch kubectl namespace"
    set --local NEW_NAMESPACE (kubectl get namespace -o custom-columns=':metadata.name' | fzf_prompt "Select namespace" "$NAMESPACE")
    if test -z "$NEW_NAMESPACE"
        echo "kcc: No namespace selected"
        return 1
    end

    kubectl config set-context --current --namespace="$NEW_NAMESPACE"
end
