function fkc --description "Fuzzy kubectl context switch"
    set -l ctx (kubectl config get-contexts -o name | fzf --query "$argv")
    and kubectl config use-context $ctx
end
