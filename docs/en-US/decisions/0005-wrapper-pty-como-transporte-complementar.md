# ADR-0005 — PTY wrapper as a complementary, optional transport

> Translation of [`../../pt-BR/decisions/0005-wrapper-pty-como-transporte-complementar.md`](../../pt-BR/decisions/0005-wrapper-pty-como-transporte-complementar.md),
> which is the source of truth.

**Status:** accepted — 2026-08-23
**C4 level:** [container](../architecture/02-container.md)
**Specs this decision moves:** [capture-transports](../specs/capture-transports.md), [speech-input](../specs/speech-input.md)

## Context

Two things the person asked for do not fit in any hook:

1. Narrating the assistant's text as it is being written. Only Claude Code has an
   event for that (`MessageDisplay`); Codex and Copilot do not.
2. Delivering the dictated answer to the CLI. The TUI has no programmatic input,
   and simulating keystrokes at the OS level is fragile on Wayland and requires
   Accessibility permission on macOS.

A pseudo-terminal solves both: the wrapper *is* the terminal from the CLI's point
of view, so it sees everything that comes out and writes into what goes in — the
same in the system terminal, in VS Code's integrated terminal, and in IntelliJ's.

## Decision

There is a PTY wrapper (`cvb wrap -- <cli>`), **optional and enabled per CLI** in
the configuration. It is never the only source of a moment a hook already covers;
in deduplication, the hook wins. What only it covers — narration and text
injection — degrades with a visible warning when parsing fails, never in silence.

Parsing rules are versioned alongside the detected CLI version, and
`cvb doctor --pty` tests the rules against the installed version.

## Consequences

**Good.** Continuous narration across all three CLIs. Text injection without
simulating keystrokes and without special OS permission. Works the same in any
terminal, including IDE-integrated ones.

**Bad.** TUI parsing breaks on every redesign — recurring maintenance, with no
warning from the vendor. It changes how the CLIs are opened. And the wrapper
sits in the middle of everything: a defect there disturbs the whole session, not
just the voice.

**Constrains.** Transparency is mandatory: everything that goes in comes out,
byte for byte, including control sequences, resizing, and signals. A wrapper that
"improves" the output is a broken wrapper.

## Alternatives

**Simulating keystrokes at the OS level.** Rejected as the main path: Wayland
restricts it, macOS demands Accessibility, and it depends on window focus. It
remains available as the lowest-confidence path.

**Clipboard, and the person pastes.** That is the dictation default, precisely
because it never breaks — but it does not solve narration and is not fluid.

**ACP only.** It solves both cleanly, at the cost of the TUI. It is a parallel
mode, not a replacement.
