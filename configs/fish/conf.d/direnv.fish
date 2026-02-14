if not status is-interactive
    return
end

set -gx DIRENV_LOG_FORMAT ""

direnv hook fish | source
