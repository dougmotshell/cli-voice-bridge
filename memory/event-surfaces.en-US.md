> Translation of [`event-surfaces.md`](event-surfaces.md), the source of truth.

# Event surfaces of the AI CLIs

Verified on **2026-08-23**, on this machine. The detailed map is in
`docs/en-US/specs/event-normalization.md`; what is here is only what is easy to
forget.

## Versions checked

| CLI | Version |
|---|---|
| Claude Code | 2.1.241 |
| Codex CLI | 0.147.0 |
| Copilot CLI | 1.0.80 |

## What is easy to forget

**All three use different spellings.** Claude: event in `PascalCase`, payload in
`snake_case`. Copilot: `camelCase` in both, and it accepts two event spellings.
Codex's `notify`: `kebab-case` (`last-assistant-message`) — the only place in the
system with that.

**Codex stores a `trusted_hash` of the hook command** in `config.toml`, under the
`[hooks.state]` section. Changing the command leaves the hook inert until the
person confirms in a session. This will look like "the event does not fire" at
least once.

**This machine already has third-party hooks:** `rtk hook claude` on
`PreToolUse`, in Claude and Codex. Any installer must compose, never replace.

**Copilot has no "text displayed" event.** Continuous narration there comes only
from PTY or `--output-format json`.

**Copilot's `ask_user` tool** is how the agent asks questions. It arrives as
`preToolUse` with `toolName == "ask_user"` — that is `input.needed`, not
`tool.started`. Easy to classify wrong.

## Not verified yet

- Whether Codex really only fires tool hooks for Bash (third-party report, not
  confirmed on this version).
- About half of Claude Code's ~30 events.

Use the `map-cli-events` skill to close those gaps — and update this page with
the version you verified on.
