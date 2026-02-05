function knsh --description "SSH into a node via gcloud"
    argparse 'c/context=' 'zone=' 'project=' -- $argv
    or return

    set -l kubectl_flags
    if set -q _flag_context
        set -a kubectl_flags --context $_flag_context
    end

    set -l gcloud_flags
    if set -q _flag_zone
        set -a gcloud_flags --zone $_flag_zone
    end
    if set -q _flag_project
        set -a gcloud_flags --project $_flag_project
    end

    set -l node (kubectl $kubectl_flags get nodes -o name | sed 's|node/||' | fzf --select-1 --query "$argv")
    and gcloud compute ssh $gcloud_flags $node
end
