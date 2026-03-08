#!/usr/bin/env bash
# safe-text.sh -- Write text to a temp file, enforcing ASCII.
#
# Usage:
#   safe-text.sh [OPTIONS] [--file PATH | CONTENT...]
#
# Reads content from --file, positional args, or stdin (in that order).
# Replaces common non-ASCII characters with ASCII equivalents, then writes
# the result to a temp file. Prints the temp file path to stdout.
#
# Options:
#   --prefix PREFIX       Temp file prefix (default: "gh-text")
#   --file PATH           Read content from PATH instead of args/stdin
#   --commit-msg          Enable commit message rules: capitalize first
#                         letter, warn on subject >72 chars
#   --title               Enable title rules: capitalize first letter,
#                         warn on length >70 chars
#
# Exit codes:
#   0  Success (temp file path on stdout)
#   1  No content provided or content is empty after trimming

set -euo pipefail

prefix="safe-text"
source_file=""
content=""
mode=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --prefix)
      prefix="$2"
      shift 2
      ;;
    --file)
      source_file="$2"
      shift 2
      ;;
    --commit-msg)
      mode="commit-msg"
      shift
      ;;
    --title)
      mode="title"
      shift
      ;;
    *)
      if [[ -n "$content" ]]; then
        content="$content $1"
      else
        content="$1"
      fi
      shift
      ;;
  esac
done

# Read content from file, args, or stdin
if [[ -n "$source_file" ]]; then
  content="$(cat "$source_file")"
elif [[ -z "$content" ]] && [[ ! -t 0 ]]; then
  content="$(cat)"
fi

# Bail if empty
trimmed="${content#"${content%%[![:space:]]*}"}"
if [[ -z "$trimmed" ]]; then
  echo "error: no content provided" >&2
  exit 1
fi

# Replace common non-ASCII with ASCII equivalents using perl (reliable
# cross-platform Unicode handling, unlike sed on macOS)
content="$(printf '%s' "$content" | perl -CSD -pe '
  s/\x{2014}/--/g;       # em dash
  s/\x{2013}/--/g;       # en dash
  s/\x{2026}/.../g;      # ellipsis
  s/\x{2018}/\x27/g;     # left single quote
  s/\x{2019}/\x27/g;     # right single quote
  s/\x{201C}/"/g;        # left double quote
  s/\x{201D}/"/g;        # right double quote
  s/\x{00B7}/*/g;        # middle dot
  s/\x{2022}/*/g;        # bullet
  s/\x{00A0}/ /g;        # non-breaking space
  s/[^\x00-\x7F]//g;     # strip any remaining non-ASCII
')"

# Mode-specific rules applied to the first line (subject/title)
if [[ -n "$mode" ]]; then
  first_line="${content%%$'\n'*}"

  # Capitalize first letter
  if [[ "${first_line:0:1}" =~ [a-z] ]]; then
    upper="$(printf '%s' "${first_line:0:1}" | tr '[:lower:]' '[:upper:]')"
    content="${upper}${content:1}"
  fi

  # Length warnings
  local_len="${#first_line}"
  case "$mode" in
    commit-msg)
      if [[ "$local_len" -gt 72 ]]; then
        echo "warning: subject line is ${local_len} chars (limit 72)" >&2
      fi
      ;;
    title)
      if [[ "$local_len" -gt 70 ]]; then
        echo "warning: title is ${local_len} chars (limit 70)" >&2
      fi
      ;;
  esac
fi

# Write to temp file
tmpfile="$(mktemp "/tmp/${prefix}-XXXXXX")"
printf '%s\n' "$content" > "$tmpfile"

echo "$tmpfile"
