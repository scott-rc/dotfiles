# Completions for knsh (SSH into k8s node via gcloud)
complete -c knsh -f
complete -c knsh -s c -l context -d "Kubernetes context" -r -a '(__fish_knsh_contexts)'
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
    set -l flags (__kubectl_flags_from_cmdline context)
    kubectl $flags get nodes -o name 2>/dev/null | sed 's|node/||'
end
