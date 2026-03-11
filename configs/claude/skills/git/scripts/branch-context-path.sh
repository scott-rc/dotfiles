#!/usr/bin/env bash
set -euo pipefail

# Print the branch context file path for the current branch.
# Usage: branch-context-path.sh

branch=$(git rev-parse --abbrev-ref HEAD)

if [ "$branch" = "HEAD" ]; then
  echo "Not on a branch (detached HEAD)." >&2
  exit 1
fi

sanitized=${branch//\//-\-}
echo "./tmp/branches/${sanitized}/context.md"
