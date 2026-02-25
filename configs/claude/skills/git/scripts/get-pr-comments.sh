#!/usr/bin/env bash
set -euo pipefail

# Fetch unresolved PR review threads and review summaries via GitHub GraphQL API.
# Outputs structured JSON to stdout. Errors go to stderr.
#
# Usage:
#   get-pr-comments.sh              # All unresolved threads
#   get-pr-comments.sh --unreplied  # Only threads where the current user hasn't replied

unreplied=false
while [[ $# -gt 0 ]]; do
  case "$1" in
    --unreplied) unreplied=true; shift ;;
    *) echo "Unknown argument: $1" >&2; exit 1 ;;
  esac
done

if [ "$unreplied" = true ]; then
  current_user=$(gh api user --jq '.login') || {
    echo "Error: Could not determine current GitHub user." >&2
    exit 1
  }
fi

pr_json=$(gh pr view --json number,url 2>/dev/null) || {
  echo "Error: No PR found for the current branch." >&2
  exit 1
}

pr_number=$(echo "$pr_json" | jq -r '.number')
pr_url=$(echo "$pr_json" | jq -r '.url')

repo_json=$(gh repo view --json owner,name) || {
  echo "Error: Could not determine repository owner/name." >&2
  exit 1
}
owner=$(echo "$repo_json" | jq -r '.owner.login')
repo=$(echo "$repo_json" | jq -r '.name')

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
          | { id: .databaseId, author: .author.login, body: .body, createdAt: .createdAt }
        ]
      }
  ],
  reviewSummaries: [
    .data.repository.pullRequest.reviews.nodes[]
    | select(.body != null and .body != \"\")
    | { author: .author.login, body: .body, state: .state, createdAt: .createdAt }
  ]
}
"
