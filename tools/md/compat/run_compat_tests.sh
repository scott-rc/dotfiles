#!/usr/bin/env bash
#
# Language-agnostic rendering compatibility tests.
#
# Usage:
#   ./compat/run_compat_tests.sh 'deno run -A main.ts'
#   ./compat/run_compat_tests.sh './target/release/md'
#
# For each fixtures/rendering/*.md, runs the md binary with
# --no-color --no-pager --width 60 and diffs stdout against
# the corresponding .expected.txt file.

set -euo pipefail

MD_BIN="${1:?Usage: $0 <md-binary-command>}"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
FIXTURE_DIR="$SCRIPT_DIR/../fixtures/rendering"

passed=0
failed=0
errors=""

for md_file in "$FIXTURE_DIR"/*.md; do
  name="$(basename "$md_file" .md)"
  expected_file="$FIXTURE_DIR/$name.expected.txt"

  if [ ! -f "$expected_file" ]; then
    echo "SKIP: $name (no .expected.txt)"
    continue
  fi

  # Run the md binary: read from stdin via -, no color, no pager, width 60
  actual="$(eval "$MD_BIN" --no-color --no-pager --width 60 - < "$md_file" 2>&1)" || true
  expected="$(cat "$expected_file")"

  if [ "$actual" = "$expected" ]; then
    passed=$((passed + 1))
  else
    failed=$((failed + 1))
    errors="${errors}\nFAIL: $name"
    errors="${errors}\n$(diff --color=auto <(echo "$actual") <(echo "$expected") || true)"
    errors="${errors}\n"
  fi
done

echo ""
echo "Rendering compatibility: $passed passed, $failed failed"

if [ "$failed" -gt 0 ]; then
  echo -e "$errors"
  exit 1
fi
