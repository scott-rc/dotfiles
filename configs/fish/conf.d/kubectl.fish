set -gx USE_GKE_GCLOUD_AUTH_PLUGIN True

if not status is-interactive
    return
end

alias k=kubectl

abbr --add kcc kubectl config current-context
abbr --add kcn kubectl config view -o jsonpath='{..namespace}'
