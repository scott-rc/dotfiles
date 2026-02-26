#!/usr/bin/env bash
set -euo pipefail

# Poll PR state, CI checks, and review threads in a single call.
# Outputs compact JSON to stdout. Errors go to stderr.
#
# Usage:
#   poll-pr-status.sh
#   poll-pr-status.sh --last-push-time 2026-02-25T10:00:00Z
#   poll-pr-status.sh --handled-threads 123,456,789
#   poll-pr-status.sh --handled-checks check1,check2
#   poll-pr-status.sh --last-push-time 2026-02-25T10:00:00Z --handled-threads 123,456 --handled-checks check1

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

last_push_time=""
handled_threads=""
handled_checks=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --last-push-time)
      last_push_time="$2"
      shift 2
      ;;
    --handled-threads)
      handled_threads="$2"
      shift 2
      ;;
    --handled-checks)
      handled_checks="$2"
      shift 2
      ;;
    *)
      echo "Unknown argument: $1" >&2
      exit 1
      ;;
  esac
done

# --- PR state ---

pr_json=$(gh pr view --json number,state 2>/dev/null) || {
  echo '{"error":"No PR found for the current branch."}' >&2
  exit 1
}

pr_number=$(echo "$pr_json" | jq -r '.number')
pr_state=$(echo "$pr_json" | jq -r '.state')

if [[ "$pr_state" == "MERGED" ]]; then
  jq -n --argjson num "$pr_number" '{"exit":"pr_merged","pr":{"number":$num,"state":"MERGED"}}'
  exit 0
fi

if [[ "$pr_state" == "CLOSED" ]]; then
  jq -n --argjson num "$pr_number" '{"exit":"pr_closed","pr":{"number":$num,"state":"CLOSED"}}'
  exit 0
fi

# --- CI checks ---

repo_root=$(git rev-parse --show-toplevel 2>/dev/null) || repo_root=""
ci_json="null"

# Always try gh pr checks -- works for GitHub Actions, Buildkite, and any CI
# that reports check runs to GitHub.
if [[ -n "$repo_root" ]]; then
  checks_raw=$(gh pr checks --json name,state,startedAt,completedAt 2>/dev/null) || {
    echo "Warning: gh pr checks failed, treating as no CI" >&2
    checks_raw="[]"
  }

  # Detect CI system from repo structure
  ci_system="unknown"
  if [[ -d "$repo_root/.github/workflows" ]]; then
    ci_system="github-actions"
  elif [[ -d "$repo_root/.buildkite" ]]; then
    ci_system="buildkite"
  fi

  # If no checks were reported at all, treat as no CI
  if [[ "$checks_raw" == "[]" ]]; then
    ci_json="null"
  else
    ci_json=$(echo "$checks_raw" | jq \
      --arg lastPush "$last_push_time" \
      --arg ciSystem "$ci_system" \
      --arg handledChecks "$handled_checks" \
    '
      # Classify each check
      def classify:
        if .state == "SUCCESS" then "passed"
        elif .state == "FAILURE" then "failed"
        elif .state == "IN_PROGRESS" or .state == "PENDING" or .state == "QUEUED" then "pending"
        elif .state == "SKIPPED" then "skipped"
        elif .state == "CANCELLED" then "cancelled"
        elif .state == "NEUTRAL" or .state == "STALE" then "neutral"
        else "neutral"
        end;

      # Checks are actionable unless skipped, cancelled, neutral, or stale
      def is_actionable:
        .state as $s |
        ($s != "SKIPPED" and $s != "CANCELLED" and $s != "NEUTRAL" and $s != "STALE");

      # Build the set of handled check names from comma-separated string
      ($handledChecks | split(",") | map(select(. != ""))) as $handled_set |

      # A failure is "new" if:
      # - When --handled-checks is provided: the check name is NOT in the handled set
      # - When --handled-checks is omitted: falls back to timestamp filtering
      #   (started after last_push_time, or no last_push_time given)
      # The --handled-checks approach is preferred because it decouples freshness
      # from timestamps, which is critical for Buildkite where startedAt reflects
      # the original job start time, not retries.
      def is_new_failure:
        .state == "FAILURE" and
        (if ($handledChecks != "") then
          (.name | IN($handled_set[]) | not)
        else
          ($lastPush == "" or (.startedAt >= $lastPush))
        end);

      def is_pending:
        .state == "IN_PROGRESS" or .state == "PENDING" or .state == "QUEUED";

      . as $checks |
      {
        ciSystem:  $ciSystem,
        passed:    ([ $checks[] | select(.state == "SUCCESS") ] | length),
        failed:    ([ $checks[] | select(.state == "FAILURE") ] | length),
        pending:   ([ $checks[] | select(is_pending) ] | length),
        skipped:   ([ $checks[] | select(.state == "SKIPPED") ] | length),
        cancelled: ([ $checks[] | select(.state == "CANCELLED") ] | length),
        neutral:   ([ $checks[] | select(.state == "NEUTRAL" or .state == "STALE") ] | length),
        actionable: (
          if ([ $checks[] | select(is_actionable) | select(.state == "FAILURE") ] | length) > 0 then "failing"
          elif ([ $checks[] | select(is_actionable) | select(is_pending) ] | length) > 0 then "pending"
          else "passing"
          end
        ),
        newFailures: [
          $checks[] | select(is_new_failure) | { name: .name, startedAt: .startedAt }
        ],
        pendingChecks: [
          $checks[] | select(is_pending) | .name
        ]
      }
    ')
  fi
fi

# --- Review threads ---

threads_json='{"total":0,"new":0,"newThreads":[]}'
comments_raw=""

if comments_raw=$("$SCRIPT_DIR/get-pr-comments.sh" --unreplied 2>/dev/null); then
  threads_json=$(echo "$comments_raw" | jq \
    --arg handled "$handled_threads" \
  '
    # Build a set of handled IDs from the comma-separated string
    ($handled | split(",") | map(select(. != ""))) as $handled_ids |

    .unresolvedThreads as $threads |

    # A thread is identified by its last comment id
    [ $threads[]
      | (.comments | last | .id | tostring) as $tid
      | select($tid | IN($handled_ids[]) | not)
    ] as $new_threads |

    {
      total: ($threads | length),
      new: ($new_threads | length),
      newThreads: $new_threads
    }
  ')
fi

# --- Compute exit condition ---

exit_value="null"

if [[ "$ci_json" == "null" ]]; then
  # No CI -- exit is all_green if no unresolved threads
  total_threads=$(echo "$threads_json" | jq '.total')
  if [[ "$total_threads" -eq 0 ]]; then
    exit_value='"all_green"'
  fi
else
  actionable_status=$(echo "$ci_json" | jq -r '.actionable')
  total_threads=$(echo "$threads_json" | jq '.total')
  if [[ "$actionable_status" == "passing" && "$total_threads" -eq 0 ]]; then
    exit_value='"all_green"'
  fi
fi

# --- Assemble output ---

jq -n \
  --argjson prNumber "$pr_number" \
  --arg prState "$pr_state" \
  --argjson ci "$ci_json" \
  --argjson threads "$threads_json" \
  --argjson exit_val "$exit_value" \
'{
  pr: { number: $prNumber, state: $prState },
  ci: $ci,
  threads: $threads,
  exit: $exit_val
}'
