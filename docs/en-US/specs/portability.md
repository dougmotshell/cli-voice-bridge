# Spec — Portability

> Translation of [`../../pt-BR/specs/portability.md`](../../pt-BR/specs/portability.md),
> which is the source of truth.

**Capability:** actually work on Linux, macOS, and Windows.

**ADRs constraining this spec:** [ADR-0001](../decisions/0001-nucleo-em-rust-com-cliente-de-hook-separado.md),
[ADR-0002](../decisions/0002-gui-em-tauri-v2.md), [ADR-0008](../decisions/0008-ipc-por-socket-local.md).
**C4 level:** [container](../architecture/02-container.md).

## Problem

Four things in this project differ across the three systems, and none of them is
a detail: IPC, pseudo-terminal, global shortcut, and audio. Treating them as
"we'll adapt later" produces a project that only runs on one machine.

## Matrix

| Concern | Linux | macOS | Windows |
|---|---|---|---|
| IPC daemon ↔ clients | UNIX socket in `$XDG_RUNTIME_DIR` | UNIX socket | named pipe |
| Pseudo-terminal | `openpty` | `openpty` | ConPTY (Windows 10 1809+) |
| Global shortcut | X11 directly; Wayland requires the `GlobalShortcuts` portal | requires Accessibility permission, granted by the person | `RegisterHotKey` |
| Audio capture | ALSA/PipeWire | CoreAudio | WASAPI |
| Playback | same | same | same |
| Fallback voice | `espeak-ng` | `say` | SAPI |
| Configuration | `~/.config/cli-voice-bridge/` | `~/Library/Application Support/` | `%APPDATA%` |
| Autostart | user systemd unit | `launchd` LaunchAgent | Scheduled Task or the Run key |
| Hook command | `sh -c` | `sh -c` | `cmd`/`pwsh` — Copilot has separate `bash` and `powershell` fields |

## Rules

**No hardcoded paths.** Every configuration, socket, cache, and log path comes
from a function that knows all three systems. A literal `~/.config` in the code
is a defect.

**Wayland is the hard case.** Global shortcuts and keystroke simulation are
restricted on purpose. The supported path is the portal; where it does not exist,
the global shortcut does not work and `cvb doctor` **says so** instead of failing
silently. Clipboard dictation keeps working, which is why it is the default.

**Degrade out loud, not into silence.** A feature unavailable on the platform
becomes an explicit warning in `doctor` and in the GUI, with what the person can
do instead.

**Windows is not a second-class port.** ConPTY and named pipes from the start,
not later. `voice-clone` already solved the encoding part
(`reconfigure(utf-8)` on `stdout`/`stderr`) — inherit the lesson instead of
rediscovering it.

## Distribution

TODO: decide. The options on the table: per-platform binaries attached to a
release, `cargo install` for those who have Rust, and packaging the GUI with
`tauri build` (`.AppImage`/`.deb`, `.dmg`, `.msi`). Personal project: start with
binaries on a release and build instructions, without code signing.

Signing and notarization (macOS) and SmartScreen (Windows) will get in the way.
Acceptable for personal use; document the workaround in the manual instead of
pretending it does not exist.

## Test plan

TODO: write it. Without all three systems at hand, the realistic plan is: CI with
all three runners running `cargo test` and `cvb doctor --offline`, plus a manual
checklist in the manual for what needs real audio and real permissions.

## Open questions

- Distribution (above).
- Whether it is worth supporting Wayland without the portal through some
  alternative, or declaring it unsupported and moving on.
