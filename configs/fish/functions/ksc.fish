function ksc --argument-names CONTEXT --description "Switch kubectl context"
    set --local NEW_CONTEXT (kubectl config get-contexts -o name | fzf_prompt "Select context" "$CONTEXT")
    if test -z "$NEW_CONTEXT"
        echo "kcc: No context selected"
        return 1
    end

    kubectl config use-context "$NEW_CONTEXT"
end
