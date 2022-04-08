#!/Users/scott/.deno/bin/deno run --no-check --allow-read --allow-env

import { $, fs, path, log } from "./lib/deps.ts";
import { ensureSymlink, exists, args } from "./lib/mod.ts";

$.stdout = "inherit";
$.stderr = "inherit";
$.verbose = args.debug;

const encoder = new TextEncoder();
const decoder = new TextDecoder();

const paths = (() => {
  const root = path.dirname(new URL(".", import.meta.url).pathname);

  return {
    root,
    home: Deno.env.get("HOME"),
    configs: `${root}/configs`,
    scripts: `${root}/scripts`,
  };
})();

// homebrew
if (await exists("/opt/homebrew/bin/brew")) {
  log.debug("homebrew already installed");
} else {
  log.info("installing homebrew");
  await $`/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"`;
  await $`/opt/homebrew/bin/brew update --force`;
}

// fish
if (await exists("/opt/homebrew/bin/fish")) {
  log.debug("fish already installed");
} else {
  log.info("installing fish");
  await $`/opt/homebrew/bin/brew install fish`;
}

const etcShells = decoder.decode(await Deno.readFile("/etc/shells"));
if (etcShells.includes("/opt/homebrew/bin/fish")) {
  log.debug("fish already added to /etc/shells");
} else {
  log.info("adding fish to /etc/shells");
  await Deno.writeFile("/etc/shells", encoder.encode(etcShells.concat("\n", "/opt/homebrew/bin/fish")));
}

await ensureSymlink(`${paths.configs}/fish`, `${paths.home}/.config/fish`);

// nix
if (await exists("/nix/var/nix/profiles/default/etc/profile.d/nix-daemon.sh")) {
  log.debug("nix already installed");
} else {
  log.info("installing nix");
  await $`/bin/bash -c "$(curl -fsSL https://nixos.org/nix/install)"`;
}

await ensureSymlink(`${paths.configs}/nix/nix.conf`, `/etc/nix/nix.conf`);

// git
await ensureSymlink(`${paths.configs}/git/.gitconfig`, `${paths.home}/.gitconfig`);

// karabiner
await fs.ensureDir(`${paths.home}/.config/karabiner`);
await ensureSymlink(`${paths.configs}/karabiner/karabiner.json`, `${paths.home}/.config/karabiner/karabiner.json`);

// vim
await fs.ensureDir(`${paths.home}/.config/nvim`);
await ensureSymlink(`${paths.configs}/vim/.vimrc`, `${paths.home}/.vimrc`);
await ensureSymlink(`${paths.configs}/vim/.ideavimrc`, `${paths.home}/.ideavimrc`);
await ensureSymlink(`${paths.configs}/vim/init.vim`, `${paths.home}/.config/nvim/init.vim`);

// zsh
await ensureSymlink(`${paths.configs}/zsh/.zshrc`, `${paths.home}/.zshrc`);

// starship
await ensureSymlink(`${paths.configs}/starship/starship.toml`, `${paths.home}/.config/starship.toml`);

// nushell
await ensureSymlink(`${paths.configs}/nu`, `${paths.home}/.config/nu`);
await ensureSymlink(`${paths.configs}/nu`, `${paths.home}/Library/Application Support/nushell`);

log.info("all done");
