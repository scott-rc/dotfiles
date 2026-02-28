#!/usr/bin/env bash
set -euo pipefail

# Retrieve failed CI run database IDs for a branch via `gh run list`.
# Outputs a JSON array to stdout. Errors go to stderr.
#
# Includes runs with status=failure AND runs with status=in_progress that
# have at least one job with conclusion=failure. The latter catches cases
# where one job has already failed while another is still running, making
# the overall run appear "in_progress" to --status failure queries.
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

# Query runs with status=failure
failed_runs=$(gh run list \
  --branch "$branch" \
  --status failure \
  --limit 10 \
  --json databaseId,workflowName,name,headSha,conclusion,createdAt) || {
  echo "Error: Failed to list runs for branch '$branch'." >&2
  exit 1
}

# Query runs with status=in_progress and check their jobs for failures.
# A run can be in_progress (e.g., test job still running) while one of its
# jobs (e.g., lint) has already concluded with failure.
in_progress_runs=$(gh run list \
  --branch "$branch" \
  --status in_progress \
  --limit 10 \
  --json databaseId,workflowName,name,headSha,conclusion,createdAt) || {
  echo "Error: Failed to list in-progress runs for branch '$branch'." >&2
  exit 1
}

# Filter in_progress_runs by head-sha and check name, then probe each for
# failed jobs.
in_progress_with_failures="[]"

while IFS= read -r run; do
  run_id=$(echo "$run" | jq -r '.databaseId')
  jobs_json=$(gh run view "$run_id" --json jobs 2>&1 || echo '{"jobs":[]}')
  has_failed=$(echo "$jobs_json" | jq '[.jobs[] | select(.conclusion == "failure")] | length > 0')
  if [[ "$has_failed" == "true" ]]; then
    in_progress_with_failures=$(echo "$in_progress_with_failures" | jq --argjson run "$run" '. + [$run]')
  fi
done < <(echo "$in_progress_runs" | jq -c \
  --arg headSha "$head_sha" \
  --arg check "$check" \
  '.[] | select($headSha == "" or .headSha == $headSha) | select($check == "" or (.name | ascii_downcase | contains($check | ascii_downcase)) or (.workflowName | ascii_downcase | contains($check | ascii_downcase)))')

# Filter failed_runs by head-sha and check name (mirrors the in_progress filter above)
filtered_failed=$(echo "$failed_runs" | jq \
  --arg headSha "$head_sha" \
  --arg check "$check" \
  '[.[] | select($headSha == "" or .headSha == $headSha) | select($check == "" or (.name | ascii_downcase | contains($check | ascii_downcase)) or (.workflowName | ascii_downcase | contains($check | ascii_downcase)))]')

# Merge filtered_failed and in_progress_with_failures, deduplicate by databaseId
all_runs=$(jq -n \
  --argjson failed "$filtered_failed" \
  --argjson in_progress "$in_progress_with_failures" \
  '$failed + $in_progress | unique_by(.databaseId)')

# Reshape only — both inputs are already filtered
echo "$all_runs" | jq \
'
  [ .[]
    | {
        runId: .databaseId,
        workflowName: .workflowName,
        headSha: .headSha,
        createdAt: .createdAt
      }
  ]
'
