#!/usr/bin/env bash
set -euo pipefail

# Check CI status for the current branch and report a grouped summary.
# Usage: check-ci.sh

# Verify a PR exists
pr_json=$(gh pr view --json number,url 2>/dev/null) || {
  echo "No PR found for the current branch."
  exit 0
}

pr_url=$(echo "$pr_json" | jq -r '.url')
echo "PR: $pr_url"
echo

# Fetch CI checks
checks_raw=$(gh pr checks --json name,state 2>/dev/null) || checks_raw="[]"

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
