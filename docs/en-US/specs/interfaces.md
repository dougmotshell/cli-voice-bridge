# Spec — Interfaces: CLI and GUI

> Translation of [`../../pt-BR/specs/interfaces.md`](../../pt-BR/specs/interfaces.md),
> which is the source of truth.

**Capability:** operate everything from the command line and everything from a
graphical interface, with real parity.

**ADRs constraining this spec:** [ADR-0002](../decisions/0002-gui-em-tauri-v2.md),
[ADR-0008](../decisions/0008-ipc-por-socket-local.md).
**C4 level:** [container](../architecture/02-container.md).

## Problem

The project has to serve two different moments: configuring and observing (where
a screen wins) and automating and diagnosing (where the command line wins). Doing
only one of them makes half the usage uncomfortable; doing both without
discipline produces two tools that disagree.

## The rule

Both are clients of the same daemon, over the same IPC. Neither has its own
policy, queue, or synthesis logic. A new feature enters the daemon first; the CLI
and the GUI only expose it. That is what makes parity a consequence rather than a
promise.

## CLI — `cvb`

| Command | Does |
|---|---|
| `cvb doctor` | Full diagnostics. The first step of any investigation |
| `cvb install [--cli …] [--dry-run]` | Installs/updates hooks, composing with existing ones |
| `cvb uninstall [--cli …]` | Removes only what it installed |
| `cvb daemon [start\|stop\|status\|logs]` | `hookd` lifecycle |
| `cvb say <text>` | End-to-end test of speech output |
| `cvb listen` | Tests speech input and shows the transcription |
| `cvb wrap -- <cli> [args]` | Opens a CLI inside the PTY wrapper |
| `cvb console --cli <name>` | Protocol client mode (ACP/app-server) |
| `cvb config [show\|edit\|check]` | Configuration |
| `cvb profile [list\|use] <name>` | Profiles |
| `cvb events [--follow] [--json]` | The moment stream, for debugging and for composing with other tools |
| `cvb mute [duration]` / `cvb unmute` | Temporary silence |

`--json` on everything that produces data. Exit codes mean something: 0 success,
1 execution failure, 2 invalid configuration, 3 daemon down.

## GUI — Tauri v2

A normal window plus a tray icon, because the main use is running in the
background.

- **Live panel:** moments arriving, what is speaking now, what is queued, and a
  button to cut the speech.
- **Microphone indicator**, unmistakable while recording.
- **Configuration:** the same keys as the TOML, validated as you type.
- **Voices:** lists the voices from `voice-clone` and lets you hear a sample.
- **Diagnostics:** `cvb doctor` as a screen, with what is broken in red and what
  to do about it.
- **Tray:** mute, switch profile, open the window, quit.

TODO: decide the front-end framework. There is no strong reason for anything
heavy; the surface is small. Evaluate going frameworkless before importing one.

## Where the two deliberately diverge

- `cvb wrap` does not exist in the GUI: it is a terminal wrapper, and a terminal
  inside a graphical window would be a different product.
- The tray indicator does not exist in the CLI: `cvb daemon status` is the
  equivalent.

Any other divergence is a defect.

## Alternatives considered

**CLI only.** Rejected: configuring dozens of keys and watching a live queue is
uncomfortable in plain text. **GUI only.** Rejected: it kills diagnosis over
`ssh` and automation. **A TUI instead of a GUI.** Considered; it loses the tray
icon, which is exactly what a background process needs.

## Test plan

TODO: write it. Minimum: a test that enumerates the CLI commands and the GUI
actions and fails when something exists on only one side without being in the
deliberate-divergence list above.

## Open questions

- The GUI front-end (above).
- Whether the GUI should be able to start the daemon itself or require
  `cvb daemon start`.
