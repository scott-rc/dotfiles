# Source nix-daemon.fish if it exists (native fish support from Nix)
# This avoids using bass which requires Python and can fail if direnv
# modifies PATH before this script runs.
set -l nix_fish_script "/nix/var/nix/profiles/default/etc/profile.d/nix-daemon.fish"
if test -f $nix_fish_script
    source $nix_fish_script
end
