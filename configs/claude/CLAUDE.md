# User Preferences

## Timezone

- Assume the user is in **Eastern Time (ET)** — America/Toronto.
- When displaying or printing dates and times, MUST use ET (EST/EDT as seasonally appropriate).

## Path Resolution

- Always resolve `tmp/` as `./tmp/` relative to the working directory, not as `/tmp/`.

## Repository Map

All git repos live under `~/Code/{personal,gadget,scratch}/<name>`.

When the user references a repo by name (e.g., "check gadget", "look at the skill in dotfiles"), use look in the above directories to resolve the path.
