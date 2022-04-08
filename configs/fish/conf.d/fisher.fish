if not status is-interactive
    return
end

if not functions --query fisher
    echo 'installing fisher'
    curl --fail --silent --show-error --location https://git.io/fisher | source && fisher install jorgebucaran/fisher
end

if not functions --query bass
    echo 'installing bass'
    fisher install edc/bass
end
