#!/usr/bin/env bash
set -euo pipefail

# Print the branch context file path.
# Usage: branch-context-path.sh [--branch NAME]

branch=""
while [[ $# -gt 0 ]]; do
  case "$1" in
    --branch) branch="$2"; shift 2 ;;
    *) echo "Unknown argument: $1" >&2; exit 1 ;;
  esac
done

if [[ -z "$branch" ]]; then
  branch=$(git rev-parse --abbrev-ref HEAD)
  if [ "$branch" = "HEAD" ]; then
    echo "Not on a branch (detached HEAD)." >&2
    exit 1
  fi
fi

sanitized=${branch//\//-\-}
echo "./tmp/branches/${sanitized}/context.md"
