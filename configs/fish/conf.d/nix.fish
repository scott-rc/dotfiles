fish_add_path "/nix/store/25xpcw4nc7wnb4fmd15fxai6wazccny3-nixpkgs-fmt-1.3.0/bin/"

if not status is-interactive
    return
end

bass source "/nix/var/nix/profiles/default/etc/profile.d/nix-daemon.sh"
