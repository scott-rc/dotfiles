#!/usr/bin/env bash
# sanitize.sh -- In-place ASCII text sanitizer.
#
# Usage:
#   sanitize.sh [--commit-msg | --title] <file>
#
# Operates in-place on the given file. Replaces common non-ASCII characters
# with ASCII equivalents and strips any remaining non-ASCII.
#
# Options:
#   --commit-msg          Enable commit message rules: capitalize first
#                         letter, error if subject >72 chars
#   --title               Enable title rules: capitalize first letter,
#                         error if length >70 chars
#
# Exit codes:
#   0  Success (file modified in place)
#   1  Content empty, subject/title exceeds length limit, or no file given

set -euo pipefail

mode=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --commit-msg)
      mode="commit-msg"
      shift
      ;;
    --title)
      mode="title"
      shift
      ;;
    *)
      break
      ;;
  esac
done

if [[ $# -lt 1 ]]; then
  echo "error: no file specified" >&2
  exit 1
fi

target="$1"

if [[ ! -f "$target" ]]; then
  echo "error: file not found: $target" >&2
  exit 1
fi

content="$(cat "$target")"

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

  # Length checks
  local_len="${#first_line}"
  case "$mode" in
    commit-msg)
      if [[ "$local_len" -gt 72 ]]; then
        echo "error: subject line is ${local_len} chars (limit 72)" >&2
        exit 1
      fi
      ;;
    title)
      if [[ "$local_len" -gt 70 ]]; then
        echo "error: title is ${local_len} chars (limit 70)" >&2
        exit 1
      fi
      ;;
  esac
fi

# Write atomically via temp file + mv
tmpfile="$(mktemp "${target}.tmp.XXXXXX")"
printf '%s\n' "$content" > "$tmpfile"
mv "$tmpfile" "$target"
