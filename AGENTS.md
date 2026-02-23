# AGENTS.md

@CLAUDE.md
@configs/claude/CLAUDE.md

## Authority

Claude configuration is the source of truth for shared agent behavior in this repo.

- Rules authority: `configs/claude/rules/*.md`
- Skill authority: `configs/claude/skills/*`
- Claude runtime settings authority: `configs/claude/settings.json`

## Codex Alignment

- `apply.sh` maintains Claude-backed links for Codex/Agents:
- `~/.codex/skills/*` and `~/.agents/skills/*` → `configs/claude/skills/*`
- `~/.codex/claude-rules` → `configs/claude/rules`
- Codex should treat Claude-authored markdown guidance as canonical unless overridden by Codex system/developer instructions.
