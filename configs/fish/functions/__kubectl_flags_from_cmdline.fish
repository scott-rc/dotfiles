function __kubectl_flags_from_cmdline --description "Parse kubectl flags from command line for completions"
    # Usage: set -l flags (__kubectl_flags_from_cmdline context kubeconfig namespace)
    set -l wanted_flags $argv
    set -l cmdline (commandline -opc)
    set -l result

    for i in (seq (count $cmdline))
        set -l token $cmdline[$i]
        for flag in $wanted_flags
            # Handle --flag=value syntax
            if string match -q -- "--$flag=*" $token
                set -a result --$flag (string replace -- "--$flag=" '' $token)
            # Handle --flag value syntax
            else if test "$token" = "--$flag"; and set -q cmdline[(math $i + 1)]
                set -a result --$flag $cmdline[(math $i + 1)]
            end
            # Handle -n for namespace
            if test "$flag" = namespace; and test "$token" = "-n"; and set -q cmdline[(math $i + 1)]
                set -a result -n $cmdline[(math $i + 1)]
            end
            # Handle -c for context
            if test "$flag" = context; and test "$token" = "-c"; and set -q cmdline[(math $i + 1)]
                set -a result --context $cmdline[(math $i + 1)]
            end
        end
    end
    printf '%s\n' $result
end
