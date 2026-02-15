# Commit Message Guidelines

## Structure

```
<title>

<body>
```

Separate title and body with a blank line. The body is optional for trivial changes.

## Title

- MUST keep under 72 characters (GitHub truncates longer titles)
- MUST use imperative mood: "Add feature" not "Added feature"
- SHOULD be specific: "Fix null pointer in UserService.load()" not "Fix bug"

## Body

- SHOULD explain the motivation and reasoning -- why these changes were made, not just what changed
- MUST write prose, not bullets (unless listing genuinely unrelated items)
- Use backticks for code: function names, variables, file paths, flags, etc.

## Examples

**Feature addition** (title + body):
```
Add workspace-level snippet sharing

Users in the same workspace frequently recreate identical snippets. This
introduces a shared snippet library scoped to the workspace, with
copy-on-edit semantics so personal modifications don't affect the
original. Storage uses the existing `snippets` table with an added
`workspace_id` column.
```

**Bug fix** (title + body):
```
Fix race condition in WebSocket reconnect handler

`reconnect()` was firing before the previous socket's `onclose` callback
completed, leaving a dangling reference in `activeConnections`. The fix
gates reconnection on the close callback via `socket.addEventListener`
instead of checking `readyState` in a polling loop.
```

**Simple change** (title only, no body needed):
```
Bump eslint-plugin-react from 7.33.2 to 7.34.0
```

## When Unsure

If the commit scope or message is unclear, ask the user before committing.
