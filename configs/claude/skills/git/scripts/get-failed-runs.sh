#!/usr/bin/env bash
set -euo pipefail

# Retrieve failed CI run database IDs for a branch via `gh run list`.
# Outputs a JSON array to stdout. Errors go to stderr.
#
# Usage:
#   get-failed-runs.sh                          # Failed runs on current branch
#   get-failed-runs.sh --branch my-branch       # Failed runs on a specific branch
#   get-failed-runs.sh --check "lint"            # Filter to runs whose name contains "lint"
#   get-failed-runs.sh --head-sha abc123         # Only runs for a specific commit

branch=""
check=""
head_sha=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --branch)
      branch="$2"
      shift 2
      ;;
    --check)
      check="$2"
      shift 2
      ;;
    --head-sha)
      head_sha="$2"
      shift 2
      ;;
    *)
      echo "Unknown argument: $1" >&2
      exit 1
      ;;
  esac
done

if [[ -z "$branch" ]]; then
  branch=$(git branch --show-current) || {
    echo "Error: Could not determine current branch." >&2
    exit 1
  }
fi

runs=$(gh run list \
  --branch "$branch" \
  --status failure \
  --limit 10 \
  --json databaseId,workflowName,name,headSha,conclusion,createdAt) || {
  echo "Error: Failed to list runs for branch '$branch'." >&2
  exit 1
}

echo "$runs" | jq \
  --arg headSha "$head_sha" \
  --arg check "$check" \
'
  [ .[]
    | select($headSha == "" or .headSha == $headSha)
    | select($check == "" or (.name | ascii_downcase | contains($check | ascii_downcase)) or (.workflowName | ascii_downcase | contains($check | ascii_downcase)))
    | {
        runId: .databaseId,
        workflowName: .workflowName,
        headSha: .headSha,
        createdAt: .createdAt
      }
  ]
'
