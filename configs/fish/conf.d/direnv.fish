set -gx DIRENV_LOG_FORMAT ""

if not status is-interactive
    return
end

direnv hook fish | source
