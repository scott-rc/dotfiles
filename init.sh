#!/usr/bin/env sh

if ! test -e "$HOME/.deno/bin/deno"; then
    curl -fsSL https://deno.land/install.sh | sh
fi
