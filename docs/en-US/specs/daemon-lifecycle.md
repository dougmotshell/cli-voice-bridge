# Spec — Daemon lifecycle

> Translation of [`../../pt-BR/specs/daemon-lifecycle.md`](../../pt-BR/specs/daemon-lifecycle.md),
> which is the source of truth.

**Capability:** `hookd` comes up when it is needed, dies without leaving mess,
and keeps the sidecar alive.

**ADRs constraining this spec:** [ADR-0001](../decisions/0001-nucleo-em-rust-com-cliente-de-hook-separado.md),
[ADR-0003](../decisions/0003-tts-delegado-ao-voice-clone.md),
[ADR-0008](../decisions/0008-ipc-por-socket-local.md).
**C4 level:** [container](../architecture/02-container.md).

## State

| Part | State |
|---|---|
| Single instance | **works** — `Ouvinte::abrir` refuses if someone is already answering |
| Orphan socket from a previous run | **works** — detected and removed at startup |
| Graceful shutdown | **does not exist** — the daemon dies on a signal, without draining or cleaning up |
| On-demand startup | **does not exist** — `hookc` exits silently if it finds no daemon |
| Autostart at login | **does not exist** |
| Sidecar supervision | **does not exist** — if it dies, nothing brings it back |

## Problem

A daemon that only starts when the person remembers to start it is only useful
while they remember. And a daemon that dies on a signal leaves three things
behind: the socket on disk, an undrained queue, and possibly an audio player
process still going.

None of that is serious today — the next startup cleans the socket, the queue is
volatile by nature, and the player finishes on its own. But each becomes a real
defect the moment anything depends on predictable shutdown.

## Scope

In: startup, shutdown, single instance, orphan socket, and supervision of the
synthesis sidecar.
Out: what the daemon does while alive — that is in the other specs.

## Design

### Startup

Three paths, from simplest to most automatic:

1. **By hand:** `cvb daemon start`. That is what exists today, in the form of
   running the binary. TODO: the subcommand still exits with "not implemented".
2. **On demand:** the first `hookc` that does not find the socket asks for it to
   come up. TODO: decide whether it is worth it — there is a risk of several
   hooks trying to start it at once, which the single-instance refusal already
   handles, but at the cost of processes being born and dying.
3. **At login**, depending on configuration:

| System | Mechanism |
|---|---|
| Linux | user systemd unit (`~/.config/systemd/user/`) |
| macOS | `launchd` LaunchAgent (`~/Library/LaunchAgents/`) |
| Windows | Scheduled Task at logon, or the `Run` key |

TODO: `cvb daemon install-autostart` / `uninstall-autostart`, with the same
`--dry-run` as `cvb install`. The same rule applies: never overwrite someone
else's unit.

### Graceful shutdown

On `SIGTERM` or `SIGINT` (and the Windows equivalent):

1. Stop accepting new connections.
2. **Kill the running player.** Leaving audio playing after the daemon dies is
   the worst of both worlds: nobody can cut it off.
3. Discard the queue. Do not drain: if the person is shutting down, they do not
   want to hear anything more. Whatever was critical has lost its point — the CLI
   that was waiting is going away too.
4. Close the sidecar connection without killing it: it is a separate process,
   with an owner of its own.
5. Remove the socket.

`Fila::encerrar` already exists and today only the tests use it; this is where it
belongs.

**Why `Ouvinte`'s `Drop` is not enough:** death by signal does not unwind the
stack, so `Drop` does not run and the socket stays. It is already handled
defensively at the next startup, but handling it on the way out is cleaner and
leaves the machine without residue.

### Single instance

`Ouvinte::abrir` tries connecting to the address before listening. If someone
answers, it is `AddrInUse` and the new daemon gives up; if nobody answers, the
socket is garbage from a run that died, and it is removed. That already works.

### Sidecar supervision

The sidecar is a separate Python process, with an owner of its own (ADR-0003).
The daemon does not start it today.

TODO: decide between three postures, which have quite different consequences:

- **Do not supervise** (today). The daemon falls back to the system voice and
  says so. Honest and simple, and the person has to start the sidecar by hand.
- **Restart it when it dies.** Needs an attempt limit and backoff, otherwise a
  sidecar that breaks while loading the model becomes a process loop.
- **Let the system supervisor handle it** — systemd/launchd, with the same
  mechanism as autostart. Probably the right answer: it is where that
  responsibility already lives.

## Data and contracts

No daemon state is persisted. Live sessions, queue, and muting are all volatile,
and restarting resets everything — deliberately: what matters is what is
happening now.

## Privacy

The session log has configurable retention (`privacidade.retencao_log_dias`).
TODO: retention pruning is not implemented yet; graceful shutdown is a good place
for it to run.

## Alternatives considered

**Draining the queue before exiting** instead of discarding it. Rejected: a
shutdown that takes a while is a shutdown the person kills with `-9`, and then
there was no graceful shutdown at all.

**A PID file** for single-instance. Rejected: the socket already answers "is
anyone alive?" with fewer moving parts and no stale PID.

**Having the daemon start and monitor the sidecar as a child.** Considered; it
ties the two lifecycles together and makes the daemon inherit Python's startup
problems. It comes behind letting the system supervisor handle it.

## Test plan

TODO: write it. Minimum: a test that starts two daemons on the same address and
asserts the second refuses; one that leaves an orphan socket and asserts startup
removes it; and a shutdown test asserting the socket disappears and no player
process survives.

## Open questions

- Whether on-demand startup is worth it (above).
- The sidecar supervision posture (above).
- Whether `cvb daemon start` should fork and return control, or stay in the
  foreground and leave detaching to the caller.
