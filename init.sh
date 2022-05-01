#!/usr/bin/env sh

if ! test -e ~/.deno/bin/deno; then
    curl --proto '=https' --tlsv1.2 -sSf https://deno.land/install.sh | sh
fi

"$(dirname "$0")"/scripts/setup.ts "$@"
