> Translation of [`copilot-instructions.md`](copilot-instructions.md), which is
> the source of truth and the file Copilot CLI actually reads.

# Copilot instructions — cli-voice-bridge

@AGENTS.md

`AGENTS.md` is this project's canonical contract and applies here in full. What
follows is only what is specific to Copilot CLI.

## Generated prompts

`.github/prompts/*.prompt.md` — `new-adr`, `new-spec`, `map-cli-events`,
`smoke-voice`. The sources are `skills/<name>/SKILL.md`.

## Path-scoped instructions

`.github/instructions/*.instructions.md` are generated from
`.claude/rules/*.md` (the source's `paths:` becomes `applyTo:`).

## MCP servers

None specific to this project.

## The generator rule

Never edit a file carrying the `managed-by:cli-voice-bridge/sync-ai-surfaces`
banner. Edit the source and run `python3 scripts/sync-ai-surfaces.py`.

## When touching this project's hooks

Copilot CLI hooks live in `.github/hooks/*.json` (repository) and
`~/.copilot/hooks/` (user). Event names in `camelCase` and payload in
`camelCase` — a different dialect from Claude and Codex. Read
`docs/en-US/specs/event-normalization.md` before assuming any field.
