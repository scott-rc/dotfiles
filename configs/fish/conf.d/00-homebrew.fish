# Put Homebrew on PATH (canonical `brew shellenv` setup).
# Named 00-* so brew is available to every other conf.d file (e.g. gcloud.fish).
# No `status is-interactive` guard: PATH must be set for non-interactive shells too.
if test -x /opt/homebrew/bin/brew
    eval (/opt/homebrew/bin/brew shellenv)
end
