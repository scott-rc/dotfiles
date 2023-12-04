#!/usr/bin/env bash

set -euo pipefail

if ! [[ -x "$HOME"/.deno/bin/deno ]]; then
	curl -fsSL https://deno.land/x/install/install.sh | sh
fi

PATH="$HOME/.deno/bin:$PATH"
export PATH

WORKSPACE_ROOT="$(realpath "$(dirname "$0")")"
export WORKSPACE_ROOT

"$WORKSPACE_ROOT"/scripts/setup.ts "$@"

