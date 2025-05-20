set -gx USE_GKE_GCLOUD_AUTH_PLUGIN True

if not status is-interactive
    return
end

brew_ensure kubectl kubernetes-cli

alias k=kubectl

abbr --add kcc kubectl config current-context
abbr --add kcn kubectl config view -o jsonpath='{..namespace}'

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

function knsh --argument-names NODE --description "SSH into a node"
    if test -z "$NODE"
        echo "knsh: Argument NODE is required"
        return 1
    end

    set --local NODE_NAME (kubectl get nodes -o custom-columns=':metadata.name' | grep "$NODE" | gum choose --select-if-one)
    if test -z "$NODE_NAME"
        echo "knsh: No node selected"
        return 1
    end

    echo "knsh: SSHing into node $NODE_NAME"
    gcloud compute ssh "$NODE_NAME"
end

function ksc --argument-names CONTEXT --description "Switch kubectl context"
    set --local NEW_CONTEXT (kubectl config get-contexts -o name | fzf_prompt "Select context" "$CONTEXT")
    if test -z "$NEW_CONTEXT"
        echo "kcc: No context selected"
        return 1
    end

    kubectl config use-context "$NEW_CONTEXT"
end

function ksn --argument-names NAMESPACE --description "Switch kubectl namespace"
    set --local NEW_NAMESPACE (kubectl get namespace -o custom-columns=':metadata.name' | fzf_prompt "Select namespace" "$NAMESPACE")
    if test -z "$NEW_NAMESPACE"
        echo "kcc: No namespace selected"
        return 1
    end

    kubectl config set-context --current --namespace="$NEW_NAMESPACE"
end
