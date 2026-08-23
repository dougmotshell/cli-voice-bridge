# ADR-0008 — IPC over a local socket, never a TCP port

> Translation of [`../../pt-BR/decisions/0008-ipc-por-socket-local.md`](../../pt-BR/decisions/0008-ipc-por-socket-local.md),
> which is the source of truth.

**Status:** accepted — 2026-08-23
**C4 level:** [container](../architecture/02-container.md)
**Specs this decision moves:** [capture-transports](../specs/capture-transports.md), [interfaces](../specs/interfaces.md), [portability](../specs/portability.md)

## Context

Four kinds of client talk to `hookd`: `hookc` (hundreds of times per session, has
to be fast), the CLI, the GUI, and the TTS sidecar. What travels is the content
of the work — paths, commands, fragments of the assistant's message.

## Decision

UNIX socket on Linux and macOS; named pipe on Windows. Never a TCP port, not even
on `localhost`.

The protocol is message-oriented, with a version in the handshake. A client on an
incompatible version gets an explanatory refusal instead of undefined behavior.

## Consequences

**Good.** Filesystem permissions already do the access control — no invented
authentication. Nothing listens on the network, so nothing is reachable from
outside or shows up in a port scan. A local connection is faster than the TCP
stack, which matters on the `hookc` path.

**Bad.** Two code paths, because of Windows. And the daemon is tied to the
machine: a CLI running over `ssh` on another machine cannot reach the local
daemon — that is the open question in
[capture-transports](../specs/capture-transports.md).

**Constrains.** No feature may assume remote access. If it ever becomes
necessary, that is a new, explicit transport with its own ADR and real
authentication — not a relaxation of this one.

## Alternatives

**HTTP on `localhost`.** Rejected: any process owned by the user can reach it, it
requires inventing authentication, and `voice-clone` already made the analogous
decision to listen only on `127.0.0.1` precisely because it did not want a
network surface.

**A queue file on disk.** Rejected: latency and cleanup, and it leaves the
content of the work sitting on disk.

**D-Bus.** Rejected: Linux only.
