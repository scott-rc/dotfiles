# Shell Guidelines

Covers Bash scripts (`.sh`) and Fish shell (`.fish`).

## Bash

- Shebang: `#!/usr/bin/env bash`
- MUST start with `set -euo pipefail`
- Quote all variable expansions: `"$var"`, `"${arr[@]}"`
- Use `[[ ]]` over `[ ]` for conditionals
- Use `$(command)` over backticks
- Local variables in functions: `local name="$1"`
- MUST pass shellcheck — run `shellcheck <file>` and fix all warnings before presenting code. Common issues:
  - SC2086 — unquoted variables
  - SC2046 — unquoted command substitution
  - SC2034 — unused variables
  - SC2155 — declare and assign separately (`local val; val=$(cmd)`)
  - SC2164 — use `cd ... || exit` or subshell `(cd ... && ...)`

## Fish

- No shebang needed (fish autoloads from `functions/` and `conf.d/`)
- One function per file in `functions/`, filename matches function name
- `conf.d/` files: guard interactive blocks with `status is-interactive`
- Use `set -l` for local, `set -g` for global, `set -U` for universal variables
- `test` or `[ ]` for conditionals (fish has no `[[ ]]`)
- `command -q <tool>` to check if a command exists
- `string` builtin for manipulation — prefer over `sed`/`awk`
- `$argv` for function arguments, not `$1`/`$2`
