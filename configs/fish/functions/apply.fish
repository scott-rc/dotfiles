function apply --description "Run dotfiles apply.sh"
    set -l dotfiles ~/Code/personal/dotfiles
    $dotfiles/apply.sh $argv
end
