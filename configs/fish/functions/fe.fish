function fe --description "Fuzzy find file and open in editor"
    set -l file (fzf_files --query "$argv")
    or return

    set -l repo_root (git rev-parse --show-toplevel 2>/dev/null)
    if test -n "$repo_root"
        zed $repo_root $file
    else
        zed $file
    end
end
