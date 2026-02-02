#!/bin/bash
# Wraps Bash commands with direnv exec to load environment variables

INPUT=$(cat)
COMMAND=$(echo "$INPUT" | jq -r '.tool_input.command')

# Skip if command already uses direnv
if [[ "$COMMAND" == direnv\ * ]]; then
  exit 0
fi

# Check if direnv is available and .envrc exists in the project
if command -v direnv &> /dev/null && [ -f .envrc ]; then
  # Wrap the command with direnv exec
  WRAPPED_COMMAND="direnv exec . $COMMAND"

  # Return JSON to modify the tool input
  jq -n \
    --arg cmd "$WRAPPED_COMMAND" \
    '{
      "hookSpecificOutput": {
        "hookEventName": "PreToolUse",
        "updatedInput": {
          "command": $cmd
        }
      }
    }'
fi

# No direnv or .envrc - allow command as-is
exit 0
