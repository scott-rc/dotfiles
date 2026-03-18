#!/usr/bin/env bash
set -euo pipefail

# Check CI status for a PR and report a grouped summary.
# Usage: check-ci.sh [--pr N]

pr_override=""
while [[ $# -gt 0 ]]; do
  case "$1" in
    --pr) pr_override="$2"; shift 2 ;;
    *) echo "Unknown argument: $1" >&2; exit 1 ;;
  esac
done

# Verify a PR exists
if [[ -n "$pr_override" ]]; then
  pr_json=$(gh pr view "$pr_override" --json number,url 2>/dev/null) || {
    echo "No PR found for #$pr_override."
    exit 0
  }
else
  pr_json=$(gh pr view --json number,url 2>/dev/null) || {
    echo "No PR found for the current branch."
    exit 0
  }
fi

pr_url=$(echo "$pr_json" | jq -r '.url')
pr_number=$(echo "$pr_json" | jq -r '.number')
echo "PR: $pr_url"
echo

# Fetch CI checks
if [[ -n "$pr_override" ]]; then
  checks_raw=$(gh pr checks "$pr_number" --json name,state 2>/dev/null) || checks_raw="[]"
else
  checks_raw=$(gh pr checks --json name,state 2>/dev/null) || checks_raw="[]"
fi

if [[ "$checks_raw" == "[]" ]]; then
  echo "No CI checks found."
  exit 0
fi

# Group checks into failed/pending/passed and print summary
echo "$checks_raw" | jq -r '
  def classify:
    if .state == "FAILURE" then "failed"
    elif .state == "SUCCESS" then "passed"
    elif .state == "IN_PROGRESS" or .state == "PENDING" or .state == "QUEUED" then "pending"
    elif .state == "SKIPPED" then "skipped"
    elif .state == "CANCELLED" then "cancelled"
    else "other"
    end;

  group_by(classify) | sort_by(
    .[0] | classify |
    if . == "failed" then 0
    elif . == "pending" then 1
    elif . == "passed" then 2
    else 3
    end
  ) | .[] |
  (.[0] | classify) as $status |
  "\($status | ascii_upcase) (\(length)):",
  (.[] | "  - \(.name)"),
  ""
'
