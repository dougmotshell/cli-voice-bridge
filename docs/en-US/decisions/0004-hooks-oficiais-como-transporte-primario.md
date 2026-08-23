# ADR-0004 — Official hooks as the primary transport

> Translation of [`../../pt-BR/decisions/0004-hooks-oficiais-como-transporte-primario.md`](../../pt-BR/decisions/0004-hooks-oficiais-como-transporte-primario.md),
> which is the source of truth.

**Status:** accepted — 2026-08-23
**C4 level:** [container](../architecture/02-container.md)
**Specs this decision moves:** [capture-transports](../specs/capture-transports.md), [event-normalization](../specs/event-normalization.md)

## Context

The requirement is to work in any terminal or shell — including the integrated
terminal in VS Code and IntelliJ, `tmux`, and remote sessions. All three CLIs
offer hook systems:

- Claude Code 2.1.241 — about 30 events, in `~/.claude/settings.json`
- Codex CLI 0.147.0 — `~/.codex/hooks.json` plus the `notify` key
- Copilot CLI 1.0.80 — `.github/hooks/*.json` and `~/.copilot/hooks/`

This machine already uses third-party hooks (`rtk` on `PreToolUse`, in Claude and
Codex).

## Decision

Hooks are the primary transport. Every moment a hook covers arrives via a hook.
The other transports exist for what hooks cannot reach.

Installation **composes** with what is already there: read the file, add the
entry, preserve the rest. `--dry-run` shows the diff before writing, and
uninstalling removes only what was installed.

A hook that fails exits with code 0 and in silence. The single exception is the
spoken permission decision, and even that falls back to the screen when there is
no confidence.

## Consequences

**Good.** Terminal-agnostic by construction: the one running the hook is the CLI
process. Structured JSON payload, no screen scraping. It does not break with
every TUI change.

**Bad.** Coverage is limited to what each vendor chose to expose — and the three
expose different sets, with three naming dialects. Codex keeps a `trusted_hash`
of the hook in `config.toml`, so changing the command requires the person to
confirm, and a silent installer would leave the hook inert.

**Constrains.** `hookc` has to be fast
([ADR-0001](0001-nucleo-em-rust-com-cliente-de-hook-separado.md)) and the
installer must never rewrite someone else's configuration file wholesale.

## Alternatives

**Reading the session transcript.** Rejected: asynchronous, the format is not a
public contract, and it does not say *when* the person is needed.

**PTY wrapper only.** Rejected as primary: fragile, and it forces the CLIs to be
opened differently. It became
[ADR-0005](0005-wrapper-pty-como-transporte-complementar.md).

**ACP/app-server only.** Rejected as primary: it costs the original TUI. It
remains an optional mode.
