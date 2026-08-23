> Translation of [`SKILL.md`](SKILL.md), the source of truth. The generator
> ignores this file; only `SKILL.md` is projected into the CLI surfaces.

# Map a CLI's events

Documentation says what should exist. This finds out what actually fires.

## Steps

1. **Installed version first.** `claude --version`, `codex --version`,
   `copilot --version`. Write it down — the map is valid for a version, not
   forever.

2. **Back up the hook configuration** before touching it:
   `~/.claude/settings.json`, `~/.codex/hooks.json`, `~/.copilot/hooks/`. This
   machine has third-party hooks (`rtk`) in Claude and Codex. You will put
   everything back exactly as it was.

3. **Install a temporary logging hook** — one per candidate event — that only
   dumps the raw payload to a file, with the event name, and exits with code 0.
   Nothing that could block or delay.

4. **Force each event** in a short, disposable session: ask for a permission, let
   a turn finish, spawn a subagent, provoke a tool failure. Record what arrived
   and, just as important, **what did not**.

5. **Redact before storing anything.** A real payload has home paths, project
   names, and sometimes code fragments. Replace them with placeholders before it
   becomes an example in the spec or a fixture in `crates/core/tests/fixtures/`.

6. **Update `docs/pt-BR/specs/event-normalization.md`** and its en-US sibling: the
   event → moment map, the confirmed field names, and the version you verified
   on. Whatever went unverified is marked as such — a wrong line costs more than
   a missing one.

7. **Remove the logging hooks and confirm you removed them.** Compare against the
   backup.

## Per-CLI gotchas

- **Codex** stores a `trusted_hash` of the hook command in `config.toml`:
  changing the command requires confirming in a session, otherwise the hook stays
  inert and you will wrongly conclude the event does not fire.
- **Copilot** accepts two event-name spellings and uses `camelCase` in the
  payload. Test which spelling works on the installed version.
- **Claude** uses `snake_case` in the payload and supports `matcher` — on
  `Notification`, filter by `notification_type` instead of subscribing to
  everything.
