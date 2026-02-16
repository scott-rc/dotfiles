function gpr --description "Open PR in Chrome, reusing existing tab if found"
    set -l url (gh pr view --json url -q .url 2>/dev/null)
    if test $status -ne 0
        echo "No PR found for current branch"
        return 1
    end

    # Try to find and focus an existing Chrome tab matching this PR URL
    set -l found (osascript -e '
        tell application "Google Chrome"
            set targetURL to "'"$url"'"
            repeat with w in windows
                set tabIndex to 0
                repeat with t in tabs of w
                    set tabIndex to tabIndex + 1
                    if URL of t starts with targetURL then
                        set active tab index of w to tabIndex
                        set index of w to 1
                        activate
                        return "found"
                    end if
                end repeat
            end repeat
        end tell
        return "notfound"
    ' 2>/dev/null)

    if test "$found" = found
        return 0
    end

    # No existing tab — open normally
    open $url
end
