if not status is-interactive
    return
end

set -gx MD_FIND_CMD "fd -e md -e mdx --type f . {dir}"
set -gx MD_PICK_CMD "fzf_files --scheme=path --preview 'command md --no-pager --width \$FZF_PREVIEW_COLUMNS {}'"
