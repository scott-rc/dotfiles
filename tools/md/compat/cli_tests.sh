#!/usr/bin/env bash
#
# Language-agnostic CLI behavior tests.
#
# Usage:
#   ./compat/cli_tests.sh 'deno run -A main.ts'
#   ./compat/cli_tests.sh './target/release/md'

set -euo pipefail

MD_BIN="${1:?Usage: $0 <md-binary-command>}"

passed=0
failed=0
errors=""

pass() {
  passed=$((passed + 1))
}

fail() {
  failed=$((failed + 1))
  errors="${errors}\nFAIL: $1"
}

# ── --help exits 0 and contains usage text ────────────────

output="$(eval "$MD_BIN" --help 2>&1)" && rc=$? || rc=$?
if [ "$rc" -eq 0 ] && echo "$output" | grep -qi "usage"; then
  pass
else
  fail "--help exits 0 with usage text (rc=$rc)"
fi

# ── --width 40 constrains output to 40 chars ─────────────

long_input="The quick brown fox jumps over the lazy dog and continues running through the forest."
output="$(echo "$long_input" | eval "$MD_BIN" --no-color --no-pager --width 40 - 2>&1)"
max_width=0
while IFS= read -r line; do
  len=${#line}
  if [ "$len" -gt "$max_width" ]; then
    max_width=$len
  fi
done <<< "$output"

if [ "$max_width" -le 40 ]; then
  pass
else
  fail "--width 40 constrains output (max_width=$max_width)"
fi

# ── Nonexistent file exits 1 ─────────────────────────────

eval "$MD_BIN" --no-pager /tmp/md_compat_nonexistent_file_$$ 2>/dev/null && rc=$? || rc=$?
if [ "$rc" -eq 1 ]; then
  pass
else
  fail "nonexistent file exits 1 (rc=$rc)"
fi

# ── Empty file exits 0 ───────────────────────────────────

tmpfile="$(mktemp)"
eval "$MD_BIN" --no-color --no-pager "$tmpfile" 2>/dev/null && rc=$? || rc=$?
rm -f "$tmpfile"
if [ "$rc" -eq 0 ]; then
  pass
else
  fail "empty file exits 0 (rc=$rc)"
fi

# ── Stdin pipe works ──────────────────────────────────────

output="$(echo '# Hello' | eval "$MD_BIN" --no-color --no-pager - 2>&1)" && rc=$? || rc=$?
if [ "$rc" -eq 0 ] && echo "$output" | grep -q "HELLO"; then
  pass
else
  fail "stdin pipe (rc=$rc, output=$output)"
fi

# ── Report ────────────────────────────────────────────────

echo ""
echo "CLI behavior: $passed passed, $failed failed"

if [ "$failed" -gt 0 ]; then
  echo -e "$errors"
  exit 1
fi
