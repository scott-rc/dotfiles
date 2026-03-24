#!/usr/bin/env bash
set -euo pipefail

# Fetch unresolved PR review threads and review summaries via GitHub GraphQL API.
# Outputs structured JSON to stdout. Errors go to stderr.
#
# Usage:
#   get-pr-comments.sh              # All unresolved threads (full JSON)
#   get-pr-comments.sh --unreplied  # Only threads where the current user hasn't replied
#   get-pr-comments.sh --count      # Print integer count of unresolved threads
#   get-pr-comments.sh --summary    # Print compact human-readable summary
#   get-pr-comments.sh --pr 1234    # Check a specific PR number (instead of current branch)
#
# Flags can be combined:
#   get-pr-comments.sh --unreplied --count    # Count of unreplied threads
#   get-pr-comments.sh --unreplied --summary  # Summary of unreplied threads
#   get-pr-comments.sh --pr 1234 --count      # Count for a specific PR
#
# --count and --summary are mutually exclusive; if both are passed, --summary wins.

unreplied=false
output_count=false
output_summary=false
pr_override=""
while [[ $# -gt 0 ]]; do
  case "$1" in
    --unreplied) unreplied=true; shift ;;
    --count) output_count=true; shift ;;
    --summary) output_summary=true; shift ;;
    --pr) pr_override="$2"; shift 2 ;;
    *) echo "Unknown argument: $1" >&2; exit 1 ;;
  esac
done

if [ "$unreplied" = true ]; then
  current_user=$(gh api user --jq '.login') || {
    echo "Error: Could not determine current GitHub user." >&2
    exit 1
  }
fi

repo_json=$(gh repo view --json owner,name) || {
  echo "Error: Could not determine repository owner/name." >&2
  exit 1
}
owner=$(echo "$repo_json" | jq -r '.owner.login')
repo=$(echo "$repo_json" | jq -r '.name')

if [[ -n "$pr_override" ]]; then
  pr_number="$pr_override"
  pr_url="https://github.com/$owner/$repo/pull/$pr_number"
else
  # Try git-spice local state first, fall back to gh
  branch=$(git branch --show-current 2>/dev/null)
  spice_line=$(git-spice log short --json 2>/dev/null | jq -r --arg b "$branch" 'select(.name == $b and .change != null) | "\(.change.id | ltrimstr("#"))\t\(.change.url)"') || spice_line=""
  if [[ -n "$spice_line" ]]; then
    pr_number=$(echo "$spice_line" | cut -f1)
    pr_url=$(echo "$spice_line" | cut -f2)
  else
    pr_json=$(gh pr view --json number,url 2>/dev/null) || {
      echo "Error: No PR found for the current branch." >&2
      exit 1
    }
    pr_number=$(echo "$pr_json" | jq -r '.number')
    pr_url=$(echo "$pr_json" | jq -r '.url')
  fi
fi

# $owner, $repo, $pr below are GraphQL variables, not shell
# shellcheck disable=SC2016
graphql_query='
query($owner: String!, $repo: String!, $pr: Int!) {
  repository(owner: $owner, name: $repo) {
    pullRequest(number: $pr) {
      reviewThreads(first: 100) {
        nodes {
          isResolved
          path
          line
          startLine
          diffSide
          comments(first: 50) {
            nodes {
              databaseId
              author { login }
              body
              createdAt
            }
          }
        }
      }
      reviews(first: 50) {
        nodes {
          author { login }
          body
          state
          createdAt
        }
      }
    }
  }
}'

result=$(gh api graphql -F owner="$owner" -F repo="$repo" -F pr="$pr_number" -f query="$graphql_query")

thread_filter='.data.repository.pullRequest.reviewThreads.nodes[] | select(.isResolved == false)'
if [ "$unreplied" = true ]; then
  thread_filter="$thread_filter | select((.comments.nodes | length) > 0) | select((.comments.nodes | last | .author.login) != \$me)"
fi

if [[ "$output_summary" == true ]]; then
  label="Unresolved"
  if [[ "$unreplied" == true ]]; then
    label="Unreplied"
  fi
  echo "$result" | jq -r \
    --arg me "${current_user:-}" \
    --arg label "$label" \
  "
  [
    $thread_filter
    | {
        path: .path,
        line: .line,
        comment: .comments.nodes[0]
      }
  ] | \"\(\$label): \(length)\",(
    .[]
    | .comment as \$c
    | (\$c.body
        | (capture(\"<!-- DESCRIPTION START -->(?<desc>[\\\\s\\\\S]*?)<!-- DESCRIPTION END -->\") // {desc: .}) | .desc | gsub(\"^\\\\s+|\\\\s+$\"; \"\")
        | .[0:120]) as \$body
    | \"\(.path):\(.line) — \((\$c.author.login // \"unknown\")) (id: \(\$c.databaseId)) — \(\$body)\"
  )
  "
elif [[ "$output_count" == true ]]; then
  echo "$result" | jq \
    --arg me "${current_user:-}" \
  "
  [
    $thread_filter
  ] | length
  "
else
  echo "$result" | jq \
    --argjson prNumber "$pr_number" \
    --arg prUrl "$pr_url" \
    --arg owner "$owner" \
    --arg repo "$repo" \
    --arg me "${current_user:-}" \
  "
  {
    prNumber: \$prNumber,
    prUrl: \$prUrl,
    owner: \$owner,
    repo: \$repo,
    unresolvedThreads: [
      $thread_filter
      | {
          path: .path,
          line: .line,
          startLine: .startLine,
          diffSide: .diffSide,
          comments: [
            .comments.nodes[]
            | { id: .databaseId, author: (.author.login // \"unknown\"), body: .body, createdAt: .createdAt }
          ]
        }
    ],
    reviewSummaries: [
      .data.repository.pullRequest.reviews.nodes[]
      | select(.body != null and .body != \"\")
      | { author: (.author.login // \"unknown\"), body: .body, state: .state, createdAt: .createdAt }
    ]
  }
  "
fi
