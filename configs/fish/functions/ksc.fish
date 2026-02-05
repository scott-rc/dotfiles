function ksc --description "Switch kubectl context"
    argparse 'kubeconfig=' -- $argv
    or return

    set -l kubectl_flags
    if set -q _flag_kubeconfig
        set -a kubectl_flags --kubeconfig $_flag_kubeconfig
    end

    set -l ctx (kubectl $kubectl_flags config get-contexts -o name | fzf --select-1 --query "$argv")
    and kubectl $kubectl_flags config use-context $ctx
end
