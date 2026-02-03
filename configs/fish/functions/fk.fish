function fk --description "Fuzzy kill process"
    set -l pid (ps -ef | sed 1d | fzf --query "$argv" --multi | awk '{print $2}')
    and kill $pid
end
