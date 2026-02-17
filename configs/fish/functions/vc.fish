function vc --description "Edit clipboard contents in nvim"
    set -l tmpfile (mktemp /tmp/vc-XXXXXX)
    pbpaste > $tmpfile
    $EDITOR $tmpfile
    pbcopy < $tmpfile
    rm -f $tmpfile
end
