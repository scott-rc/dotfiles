#!/usr/bin/env bash
set -euo pipefail

# Re-trigger failed CI jobs on the current branch.
# Usage: rerun.sh

branch=$(git branch --show-current)

# Find the most recent failed run
run_json=$(gh run list --branch "$branch" --status failure --limit 1 --json databaseId,workflowName 2>/dev/null) || run_json="[]"

if [[ "$run_json" == "[]" ]] || [[ $(echo "$run_json" | jq 'length') -eq 0 ]]; then
  echo "No failed runs found on branch $branch."
  exit 0
fi

run_id=$(echo "$run_json" | jq -r '.[0].databaseId')
workflow_name=$(echo "$run_json" | jq -r '.[0].workflowName')

echo "Re-running failed workflow: $workflow_name (run $run_id)"

# Attempt --failed first (re-runs only failed jobs), fall back to full rerun
if ! gh run rerun "$run_id" --failed 2>/dev/null; then
  echo "Falling back to full rerun..."
  gh run rerun "$run_id"
fi

# Report status
status=$(gh run view "$run_id" --json status -q '.status' 2>/dev/null) || status="unknown"
echo "Run $run_id status: $status"
echo
echo "Hint: use /loop 2m /git fix to monitor results."
