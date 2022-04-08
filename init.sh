#!/usr/bin/env sh

if ! test -e "$HOME/.deno/bin/deno"; then
    curl -fsSL https://deno.land/install.sh | sh
fi

"$HOME/.deno/bin/deno" run --no-check --allow-read --allow-env "$(dirname "${0}")/scripts/setup.ts"
