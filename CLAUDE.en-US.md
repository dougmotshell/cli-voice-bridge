> Translation of [`CLAUDE.md`](CLAUDE.md), which is the source of truth and the
> file Claude Code actually reads.

# CLAUDE.md — Claude Code adapter

@AGENTS.md

`AGENTS.md` is this project's canonical contract and applies here in full. What
follows is only what is specific to Claude Code.

## Available subagents

| Agent | For |
|---|---|
| `cli-event-cartographer` | Survey and verify an AI CLI's interaction events and keep `docs/pt-BR/specs/event-normalization.md` current |
| `voice-pipeline-doctor` | Diagnose the audio chain end to end: device, capture, STT, TTS sidecar, playback |

## Generated slash commands

`/new-adr`, `/new-spec`, `/map-cli-events`, `/smoke-voice`. The sources are
`skills/<name>/SKILL.md`; the files under `.claude/commands/` and
`.claude/skills/` are generated.

## MCP servers

None specific to this project. TODO: decide whether `hookd` should expose an MCP
surface so the agents themselves can query the speech queue.

## The generator rule

Never edit a file carrying the `managed-by:cli-voice-bridge/sync-ai-surfaces`
banner. Edit the source in `.claude/agents/`, `skills/`, or `.claude/rules/` and
run `python3 scripts/sync-ai-surfaces.py`.

## When touching this project's hooks

This repository installs Claude Code hooks into `~/.claude/settings.json`. That
machine already has third-party hooks there (`rtk`). Read before writing, compose
instead of replacing — see the matching pitfall in `AGENTS.md`.
