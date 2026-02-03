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
