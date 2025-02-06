#!/usr/bin/env -S deno run --allow-read --allow-env --allow-run --allow-write=tmp

import { $, assert, logger, path, workspaceRoot } from "./lib/deps.ts";
import { ensureDir, ensureSymlink } from "./lib/fs.ts";

logger.setup();

assert(Deno.env.get("HOME"), "HOME environment variable is not set");

const home = path(Deno.env.get("HOME")!);
const configs = workspaceRoot.join("configs");

// homebrew
if (await path("/opt/homebrew/bin/brew").exists()) {
  logger.debug("homebrew is already installed");
} else {
  logger.info("installing homebrew");
  await $`/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"`;
  await $`/opt/homebrew/bin/brew update --force`;
}

// fish
if (await path("/opt/homebrew/bin/fish").exists()) {
  logger.debug("fish is already installed");
} else {
  logger.info("installing fish");
  await $`/opt/homebrew/bin/brew install fish`;
}

const etcShells = await Deno.readTextFile("/etc/shells");
if (etcShells.includes("/opt/homebrew/bin/fish")) {
  logger.debug("fish is already added to /etc/shells");
} else {
  logger.info("adding fish to /etc/shells");
  await Deno.writeTextFile("/etc/shells", etcShells + "\n/opt/homebrew/bin/fish\n");
}

await ensureSymlink(configs.join("fish"), home.join(".config/fish"));

// ghosty
await ensureSymlink(
  configs.join("ghostty/config"),
  home.join("Library/Application Support/com.mitchellh.ghostty/config"),
);

// git
await ensureSymlink(configs.join("git/.gitconfig"), home.join(".gitconfig"));
await ensureSymlink(configs.join("git/.gitignore_global"), home.join(".config/git/.gitignore_global"));

// iterm2
if (
  (await $`defaults read com.googlecode.iterm2 PrefsCustomFolder`.quiet().output()) == configs.join("iterm2").toString()
) {
  logger.debug("iterm2 is already loading preference from the correct location");
} else {
  logger.info(`telling iterm2 that the preferences are located at ${configs}/iterm2`);
  await $`defaults write com.googlecode.iterm2 PrefsCustomFolder -string ${configs}/iterm2`;
}

if ((await $`defaults read com.googlecode.iterm2 LoadPrefsFromCustomFolder`.quiet().output()) === "1") {
  logger.debug("iterm2 is already loading preference from custom location");
} else {
  logger.info("telling iterm2 to load the preferences from the custom location");
  await $`defaults write com.googlecode.iterm2 LoadPrefsFromCustomFolder -bool true`;
}

// atuin
await ensureDir(home.join(".config/atuin"));
await ensureSymlink(configs.join("atuin/config.toml"), home.join(".config/atuin/config.toml"));

// karabiner
await ensureDir(home.join(".config/karabiner"));
await ensureSymlink(configs.join("karabiner/karabiner.json"), home.join(".config/karabiner/karabiner.json"));

// nix
if (await path("/nix/var/nix/profiles/default/etc/profile.d/nix-daemon.sh").exists()) {
  logger.debug("nix already installed");
} else {
  logger.info("installing nix");
  await $`/bin/bash -c "$(curl --proto '=https' --tlsv1.2 -sSf https://nixos.org/nix/install)"`;
}

await ensureSymlink(configs.join("nix/nix.conf"), path("/etc/nix/nix.conf"));

// nushell
await ensureSymlink(configs.join("nu"), home.join(".config/nu"));
await ensureSymlink(configs.join("nu"), home.join("Library/Application Support/nushell"));

// starship
await ensureSymlink(configs.join("starship/starship.toml"), home.join(".config/starship.toml"));

// vim
await ensureDir(home.join(".config/nvim"));
await ensureSymlink(configs.join("vim/.vimrc"), home.join(".vimrc"));
await ensureSymlink(configs.join("vim/.ideavimrc"), home.join(".ideavimrc"));
await ensureSymlink(configs.join("vim/init.vim"), home.join(".config/nvim/init.vim"));

// zed
await ensureSymlink(configs.join("zed/settings.json"), home.join(".config/zed/settings.json"));

// zellij
await ensureSymlink(configs.join("zellij/config.kdl"), home.join(".config/zellij/config.kdl"));

// zsh
await ensureSymlink(configs.join("zsh/.zshrc"), home.join(".zshrc"));

logger.info("done ✨");
