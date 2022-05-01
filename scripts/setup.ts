#!/usr/bin/env -S deno run --allow-read --allow-env --allow-run

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
  log.debug("homebrew is already installed");
} else {
  log.info("installing homebrew");
  await $`/bin/bash -c "$(curl --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"`;
  await $`/opt/homebrew/bin/brew update --force`;
}

// fish
if (await exists("/opt/homebrew/bin/fish")) {
  log.debug("fish is already installed");
} else {
  log.info("installing fish");
  await $`/opt/homebrew/bin/brew install fish`;
}

const etcShells = decoder.decode(await Deno.readFile("/etc/shells"));
if (etcShells.includes("/opt/homebrew/bin/fish")) {
  log.debug("fish is already added to /etc/shells");
} else {
  log.info("adding fish to /etc/shells");
  await Deno.writeFile("/etc/shells", encoder.encode(etcShells.concat("\n", "/opt/homebrew/bin/fish")));
}

await ensureSymlink(`${paths.configs}/fish`, `${paths.home}/.config/fish`);

// git
await fs.ensureDir(`${paths.home}/.config/git`);
await ensureSymlink(`${paths.configs}/git/.gitconfig`, `${paths.home}/.gitconfig`);
await ensureSymlink(`${paths.configs}/git/.gitignore_global`, `${paths.home}/.config/git/.gitignore_global`);

$.stdout = "piped";
$.stderr = "piped";

// iterm2
if ((await $o`defaults read com.googlecode.iterm2 PrefsCustomFolder`) === `${paths.configs}/iterm2`) {
  log.debug("iterm2 is already loading preference from the correct location");
} else {
  log.info(`telling iterm2 that the preferences are located at ${paths.configs}/iterm2`);
  await $`defaults write com.googlecode.iterm2 PrefsCustomFolder -string ${paths.configs}/iterm2`;
}

if ((await $o`defaults read com.googlecode.iterm2 LoadPrefsFromCustomFolder`) === "1") {
  log.debug("iterm2 is already loading preference from custom location");
} else {
  log.info("telling iterm2 to load the preferences from the custom location");
  await $`defaults write com.googlecode.iterm2 LoadPrefsFromCustomFolder -bool true`;
}

$.stdout = "inherit";
$.stderr = "inherit";

// karabiner
await fs.ensureDir(`${paths.home}/.config/karabiner`);
await ensureSymlink(`${paths.configs}/karabiner/karabiner.json`, `${paths.home}/.config/karabiner/karabiner.json`);

// nix
if (await exists("/nix/var/nix/profiles/default/etc/profile.d/nix-daemon.sh")) {
  log.debug("nix already installed");
} else {
  log.info("installing nix");
  await $`/bin/bash -c "$(curl --proto '=https' --tlsv1.2 -sSf https://nixos.org/nix/install)"`;
}

await ensureSymlink(`${paths.configs}/nix/nix.conf`, `/etc/nix/nix.conf`);

// nushell
await ensureSymlink(`${paths.configs}/nu`, `${paths.home}/.config/nu`);
await ensureSymlink(`${paths.configs}/nu`, `${paths.home}/Library/Application Support/nushell`);

// starship
await ensureSymlink(`${paths.configs}/starship/starship.toml`, `${paths.home}/.config/starship.toml`);

// vim
await fs.ensureDir(`${paths.home}/.config/nvim`);
await ensureSymlink(`${paths.configs}/vim/.vimrc`, `${paths.home}/.vimrc`);
await ensureSymlink(`${paths.configs}/vim/.ideavimrc`, `${paths.home}/.ideavimrc`);
await ensureSymlink(`${paths.configs}/vim/init.vim`, `${paths.home}/.config/nvim/init.vim`);

// zsh
await ensureSymlink(`${paths.configs}/zsh/.zshrc`, `${paths.home}/.zshrc`);

log.info("done ✨");
