fish_add_path ~/.cargo/bin
# Homebrew's rustup doesn't create proxy binaries in ~/.cargo/bin,
# so add the toolchain bin directory directly.
fish_add_path ~/.rustup/toolchains/stable-aarch64-apple-darwin/bin

if not status is-interactive
    return
end

abbr --add c cargo
abbr --add cc cargo check
abbr --add cr cargo run
abbr --add crq cargo run --quiet
