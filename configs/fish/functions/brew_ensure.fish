function brew_ensure --argument-names dependency formula --description "Ensures a brew dependency is installed"
    if test -z "$dependency"
        echo "brew_ensure: Missing required argument <dependency>"
        return 1
    end

    if command --search --quiet $dependency
        return
    end

    if test -z "$formula"
        set formula "$dependency"
    end

    echo "brew_ensure: Installing $dependency"
    command brew install "$formula" --quiet
end
