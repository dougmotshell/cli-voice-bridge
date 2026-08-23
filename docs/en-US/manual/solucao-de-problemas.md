# Troubleshooting

> Translation of [`../../pt-BR/manual/solucao-de-problemas.md`](../../pt-BR/manual/solucao-de-problemas.md),
> which is the source of truth.

**First step, always:** `cvb doctor`. It checks the whole chain and says in
Portuguese what broke. Investigate the rest only after that.

TODO: fill this in with the real problems as usage accumulates. What follows is
what is already known to be coming, from the nature of the pieces.

## It says nothing

1. `cvb daemon status` — is the daemon up? A `hookc` with no daemon exits
   silently on purpose, so it never blocks the agent.
2. `cvb doctor` — are the hooks actually installed in the CLIs marked active?
3. **On Codex:** it stores a `trusted_hash` of the hook command in `config.toml`.
   If the command changed, the hook stays inert until you confirm in a Codex
   session.
4. Are you on the `reuniao` profile or is `cvb mute` on?
5. Is the moment in question set to `falar = "nunca"` or `"ausente"` in your
   configuration?

## It speaks with the wrong voice, or a robotic one

That is the fallback: the XTTS sidecar did not come up and it fell back to the
system voice, on purpose — warning with an ugly voice beats not warning.
`cvb doctor` says why. Likely causes: the wrong `voice-clone` path in the
configuration, a broken venv over there (`falar.py checar` in `voice-clone`
settles that), or a voice name that does not exist (`falar.py vozes`).

## The first utterance takes too long

XTTS-v2 takes about 30 seconds to load. The sidecar loads once and stays alive;
if it is restarting on every utterance, that is a defect — check the log.

## It speaks too much

Adjust per moment instead of muting everything. See
[configuracao.md](configuracao.md).

**Careful: `falar = "ausente"` does not work as it should yet.** It ought to speak
only when you are not looking, but presence detection does not exist — so today
`"ausente"` simply **stays silent**. That is the documented fallback (assume
present, speak less), not a silent defect. In the meantime, use `"sempre"` for
what you want to hear and `"nunca"` for the rest. What is missing is in
[presence-detection](../specs/presence-detection.md).

## The audio cache only grows

True, and a known gap: it stores one WAV per phrase and never deletes anything.
In practice it grows slowly, because phrases repeat — but a summarized assistant
message is unique every time, and switching voice invalidates everything without
deleting it.

To clear it by hand:

| System | Directory |
|---|---|
| Linux | `~/.local/share/cli-voice-bridge/cache-audio/` |
| macOS | `~/Library/Application Support/cli-voice-bridge/cache-audio/` |
| Windows | `%LOCALAPPDATA%\cli-voice-bridge\cache-audio\` |

Deleting the directory is safe: the next utterance synthesizes again. The
automatic ceiling is still undecided — see
[speech-output](../specs/speech-output.md).

## The daemon died and left things behind

Death by signal does not run the cleanup, so the socket stays on disk. Not
serious: the next startup detects that nobody answers at that address and removes
it. If audio kept playing after the daemon died, there is no way to cut it
through `cvb` — wait it out or kill the player process.

Graceful shutdown is a known gap; what is missing is in
[daemon-lifecycle](../specs/daemon-lifecycle.md).

## It does not listen

1. `cvb listen` — shows the transcription of what it heard. If it hears nothing,
   it is the device; if it hears wrong, it is the model.
2. Right input device in the configuration? `cvb doctor` checks.
3. **The global shortcut does not work:** on Wayland it depends on the system
   portal; where that does not exist, it does not work, and `doctor` says so. Use
   clipboard dictation. On macOS, the shortcut and keystroke injection require
   Accessibility permission, granted by you in System Settings.

## It mishears "yes" and "no"

Closed-question mode uses a restricted vocabulary precisely for this. If it still
gets it wrong: speak closer, reduce noise, and check the configured STT engine.
Below the confidence threshold it does not decide — it asks again or hands the
decision back to the screen. That is correct behavior, not a defect.

## The CLI got weird after `cvb wrap`

The PTY wrapper is the fragile transport by nature: it reads the screen, and any
TUI redesign changes what it sees. `cvb doctor --pty` tests the rules against the
installed CLI version. Until there is a fix, turn `pty` off for that CLI in the
configuration and open the CLI directly — the hooks keep working on their own.

## The agent got slow

It should not: `hookc` exits in milliseconds and never waits for the daemon.
Measured at 1.94 ms per invocation. If it got slow after installing, that is a
serious defect — open an issue with the log and disable the hooks
(`cvb uninstall`) meanwhile.
