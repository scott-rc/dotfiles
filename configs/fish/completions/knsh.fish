# Completions for knsh (SSH into k8s node via gcloud)
complete -c knsh -f
complete -c knsh -l context -d "Kubernetes context" -r -a '(__fish_knsh_contexts)'
complete -c knsh -l zone -d "GCP zone" -r -a '(__fish_knsh_zones)'
complete -c knsh -l project -d "GCP project" -r -a '(__fish_knsh_projects)'
complete -c knsh -a '(__fish_knsh_nodes)'

function __fish_knsh_contexts
    kubectl config get-contexts -o name 2>/dev/null
end

function __fish_knsh_zones
    gcloud compute zones list --format='value(name)' 2>/dev/null
end

function __fish_knsh_projects
    gcloud projects list --format='value(projectId)' 2>/dev/null
end

function __fish_knsh_nodes
    set -l ctx_flag
    set -l cmdline (commandline -opc)
    if set -l idx (contains -i -- --context $cmdline)
        set ctx_flag --context $cmdline[(math $idx + 1)]
    end
    kubectl $ctx_flag get nodes -o name 2>/dev/null | sed 's|node/||'
end
