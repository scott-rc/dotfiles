#!/usr/bin/env bash
set -euo pipefail

# Fetch unresolved PR review threads and review summaries via GitHub GraphQL API.
# Outputs structured JSON to stdout. Errors go to stderr.

pr_json=$(gh pr view --json number,url 2>/dev/null) || {
  echo "Error: No PR found for the current branch." >&2
  exit 1
}

pr_number=$(echo "$pr_json" | jq -r '.number')
pr_url=$(echo "$pr_json" | jq -r '.url')

repo_json=$(gh repo view --json owner,name)
owner=$(echo "$repo_json" | jq -r '.owner.login')
repo=$(echo "$repo_json" | jq -r '.name')

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

echo "$result" | jq --argjson prNumber "$pr_number" --arg prUrl "$pr_url" '{
  prNumber: $prNumber,
  prUrl: $prUrl,
  unresolvedThreads: [
    .data.repository.pullRequest.reviewThreads.nodes[]
    | select(.isResolved == false)
    | {
        path: .path,
        line: .line,
        startLine: .startLine,
        diffSide: .diffSide,
        comments: [
          .comments.nodes[]
          | { author: .author.login, body: .body, createdAt: .createdAt }
        ]
      }
  ],
  reviewSummaries: [
    .data.repository.pullRequest.reviews.nodes[]
    | select(.body != null and .body != "")
    | { author: .author.login, body: .body, state: .state, createdAt: .createdAt }
  ]
}'
