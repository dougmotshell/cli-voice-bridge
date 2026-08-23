# Spec — Capture transports

> Translation of [`../../pt-BR/specs/capture-transports.md`](../../pt-BR/specs/capture-transports.md),
> which is the source of truth.

**Capability:** receive what the AI CLIs have to say, in any terminal, shell, or
IDE, without depending on any of them in particular.

**ADRs constraining this spec:** [ADR-0004](../decisions/0004-hooks-oficiais-como-transporte-primario.md),
[ADR-0005](../decisions/0005-wrapper-pty-como-transporte-complementar.md),
[ADR-0008](../decisions/0008-ipc-por-socket-local.md).
**C4 level:** [container](../architecture/02-container.md).

## Problem

The person uses these CLIs in different places: the system terminal, VS Code's
integrated terminal, IntelliJ's, inside `tmux`, over `ssh`, in PowerShell. A
mechanism that depends on reading the screen, on a specific emulator, or on being
in the foreground fails in most of those places.

## The choice that solves it

**Hooks run in the CLI process, not in the terminal.** When Claude Code fires
`PermissionRequest`, the one executing the hook command is Claude's own process —
it makes no difference whether it was opened in Windows Terminal, in IntelliJ's
panel, or in an `ssh` session with no TTY. That is why the hook is the primary
transport: it is the only one that is terminal-agnostic by construction.

The corollary is that the hook's output **cannot** be the speech. The hook only
hands the event to a daemon that is already running; the one who speaks is the
daemon, with the desktop session's audio device. See
[ADR-0001](../decisions/0001-nucleo-em-rust-com-cliente-de-hook-separado.md).

## The five transports

| Transport | Coverage | Confidence | Depends on the terminal? |
|---|---|---|---|
| `hook` | structured events from all 3 CLIs | high | no |
| `notify` | end of turn only, Codex only | high | no |
| `stream-json` | everything, non-interactive mode only (`-p`) | high | no |
| `acp` | everything, but replaces the TUI | high | not applicable |
| `pty` | everything, including what only exists on screen | low | yes, it is the terminal |

### hook — primary

Installed by `cvb install`. One command per event pointing at `hookc`.

- **Claude Code:** `~/.claude/settings.json` → `hooks.<Event>[].hooks[]`, type
  `command`. Accepts `matcher`, `timeout`, `async`. Supports `${CLAUDE_PROJECT_DIR}`.
- **Codex CLI:** `~/.codex/hooks.json` (or `[hooks]` in `config.toml`), plus the
  `notify` key for end of turn. Codex stores a `trusted_hash` of the hook in
  `config.toml` — changing the command requires re-confirmation; the installer
  must warn about that instead of leaving the hook silently inert.
- **Copilot CLI:** `~/.copilot/hooks/*.json` (user) and `.github/hooks/*.json`
  (repository), with `{"version": 1, "hooks": {…}}`.

**Compose, do not replace.** This machine already has `rtk hook claude` on
`PreToolUse` in Claude and Codex. `cvb install` reads the file, appends the `cvb`
entry, and preserves the rest; `cvb install --dry-run` shows the diff first.
Uninstalling removes only what it installed. TODO: decide how to mark the `cvb`
entries so they can be recognized later — JSON has no comments, so probably a
distinctive binary path.

### notify — complementary, Codex only

`notify = ["<path to hookc>", "--origem", "codex", "--transporte", "notify"]` in
`config.toml`. Codex appends the JSON as the final argument. Redundant with the
`Stop` hook; it serves as a safety net and disappears in deduplication.

### stream-json — non-interactive mode

`claude -p --output-format stream-json`, `codex exec --json`,
`copilot -p --output-format json`. Useful for background agents and scheduled
tasks, where there is no TUI at all. It does not replace hooks in interactive use.

### acp — agent protocol

Copilot CLI exposes `--acp` (Agent Client Protocol); Codex has `app-server` and
`mcp-server`; Claude Code has the Agent SDK and `--input-format stream-json`. It
is the cleanest path for two-way voice, because the spoken answer enters as a
protocol message instead of simulated keystrokes. The cost is giving up the
original TUI. It stays an optional mode: `cvb console --cli copilot`.

TODO: verify feature parity. An agent over ACP may not expose everything the TUI
does (permission modes, `/commands`, plugins).

### pty — what only exists on screen

`cvb wrap -- claude` opens the CLI inside a pseudo-terminal and passes everything
through, both ways, transparently. That captures what no hook delivers: the text
the assistant is writing right now, the rendering of the permission menu, the
question with its options. And it is what makes it possible to **inject** the
spoken answer straight into the CLI's `stdin`, without simulating keystrokes at
the OS level.

It works in any terminal because the wrapper *is* the terminal from the CLI's
point of view. It works in VS Code's and IntelliJ's integrated terminals just the
same, as long as the person opens the CLI through the wrapper.

It is fragile by nature: any TUI redesign changes what it sees. That is why it is
optional, enabled per CLI in the configuration, and **never** the only source of
a moment a hook already covers. What only it covers — narration and text
injection — degrades with a warning when parsing fails, not in silence.

TODO: decide the parsing strategy. Likely: `vte`/`anstyle` to reconstruct the
logical screen, plus per-CLI rules versioned alongside the detected version
number, with a `cvb doctor --pty` that tests the rules against the installed
version.

## Reach per operating system

| | Linux | macOS | Windows |
|---|---|---|---|
| hook | yes | yes | yes (command via `cmd`/`pwsh`) |
| pty | `openpty` | `openpty` | ConPTY |
| IPC socket | UNIX socket | UNIX socket | named pipe |
| global shortcut | `xdg-desktop-portal` (Wayland) / X11 | needs Accessibility permission | `RegisterHotKey` |

See [portability](portability.md).

## Alternatives considered

**Reading the transcript file.** Claude exposes `transcript_path` and Codex has
`~/.codex/sessions/`. Rejected as the primary transport: it is asynchronous, the
format is not a public contract, and it does not say *when* the person is needed
— which is exactly the information this project exists to provide.

**Watching terminal output through OS accessibility.** Rejected: it needs
invasive permissions, breaks in a remote terminal, and does not work over `ssh`.

**ACP only, no TUI.** Rejected as the default: the person loses the TUIs they
already use. It remains available as a mode.

## Test plan

TODO: write it. At minimum it must cover: installing a hook over a
`settings.json` that already has third-party hooks (assertion: the other hook is
still there); uninstalling (assertion: byte-for-byte return to the previous
state); and a smoke test per CLI that fires a real event and confirms it reached
the daemon.

## Open questions

- Real coverage of Codex's tool hooks — see
  [event-normalization](event-normalization.md).
- How the daemon finds the right audio device when the CLI session is on one
  machine and the person is on another (`ssh`). Likely: forward the event to the
  local machine's daemon over an explicit transport, rather than guessing.
