# Save original TMPDIR BEFORE direnv can change it
# (nix-shell flakes via direnv change TMPDIR, breaking zellij socket discovery)
# This file is named 00-* to ensure it loads before direnv.fish
if not set -q __zellij_original_tmpdir
    set -gx __zellij_original_tmpdir "$TMPDIR"
end

# Wrapper function to run zellij with the original TMPDIR
# This ensures zellij can find its sockets even after nix-shell changes TMPDIR
function zellij --wraps=zellij
    TMPDIR="$__zellij_original_tmpdir" command zellij $argv
end
