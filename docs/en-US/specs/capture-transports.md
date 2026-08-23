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
entry, and preserves the rest; `cvb install --dry-run` shows what would change and
`--diff` shows it line by line. Uninstalling removes only what it installed, and
each file's original is kept as `*.cvb-backup`.

**How an entry is recognized as ours:** its command mentions the `cvb-hook`
binary. JSON has no comments, so there is no other way to stamp it — and that is
what `cvb uninstall` uses. Reinstalling removes the previous entry before adding
the new one, so it never duplicates.

**Key order is preserved.** `serde_json` runs with `preserve_order`; without it,
rewriting `settings.json` would scramble the configuration of whoever was already
there. Content is preserved, but formatting is normalized — hence the backup.

**Copilot needs no composition.** `~/.copilot/hooks/` is a directory of JSON
files, so `cvb` writes its own (`cli-voice-bridge.json`) and never touches a
third party's. Installing is writing; uninstalling is deleting.

### Which events are subscribed

| CLI | Events |
|---|---|
| Claude | `PermissionRequest`, `Notification` (matcher `permission_prompt\|idle_prompt`), `Elicitation`, `Stop`, `StopFailure`, `SubagentStart`, `SubagentStop`, `TaskCompleted`, `PostToolUseFailure`, `SessionStart`, `SessionEnd` |
| Codex | `PermissionRequest`, `Stop`, `SubagentStart`, `SubagentStop`, `UserPromptSubmit`, `SessionStart`, `SessionEnd` |
| Copilot | `permissionRequest`, `notification`, `agentStop`, `subagentStart`, `subagentStop`, `postToolUseFailure`, `errorOccurred`, `preToolUse` (matcher `ask_user`), `sessionStart`, `sessionEnd` |

**`PreToolUse` and `PostToolUse` are deliberately left out on Claude and Codex.**
They are silent moments by default, and `PreToolUse` is the hot path where `rtk`
already lives — subscribing to both would cost on every tool call in order to say
nothing. Anyone who wants narration turns it on in the configuration and adds the
hook by hand.

On Copilot, `preToolUse` goes in **with matcher `ask_user`**, because that is how
that agent asks questions: without it, the moment the person is needed would go
unnoticed.

### notify — complementary, Codex only

`notify = ["<path to hookc>", "--origem", "codex", "--transporte", "notify"]` in
`config.toml`. Codex appends the JSON as the final argument. `hookc` already
accepts that form (payload in the argument instead of stdin).

**`cvb install` does not touch this.** Editing Codex's `config.toml` would require
a TOML editor that preserves formatting and comments — the file has dozens of
project entries, MCP servers, and hook state, and rewriting it with an ordinary
serializer would erase the formatting of whoever wrote it. Since `notify` is
redundant with the `Stop` hook, the cost does not pay for itself yet.

TODO: if it goes in, it goes in with `toml_edit`, and with the same `--dry-run`
as the rest.

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
