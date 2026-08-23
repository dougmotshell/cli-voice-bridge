# ADR-0001 — Rust core, with the hook client separate from the daemon

> Translation of [`../../pt-BR/decisions/0001-nucleo-em-rust-com-cliente-de-hook-separado.md`](../../pt-BR/decisions/0001-nucleo-em-rust-com-cliente-de-hook-separado.md),
> which is the source of truth.

**Status:** accepted — 2026-08-23
**C4 level:** [container](../architecture/02-container.md)
**Specs this decision moves:** [capture-transports](../specs/capture-transports.md), [portability](../specs/portability.md)

## Context

The hooks of all three CLIs run a command **in series with the agent**: while the
hook has not returned, the agent waits. `PreToolUse` fires on every tool call —
hundreds of times in a working session. Any fixed startup cost shows up
multiplied, and it shows up as the AI tool being slow, not as this project being
slow.

At the same time, the real work — loading an STT model, keeping a speech queue,
talking to the XTTS sidecar — is expensive and needs state between events. It
does not fit in a process that is born and dies on every hook.

The interpreter choice was measured in `voice-clone`: Python startup with that
project's imports lands in the tens of milliseconds, before any useful work.

## Decision

Two pieces, not one:

- **`hookc`** — a tiny Rust binary with no heavy dependencies. It reads the
  payload from stdin (or from the argument, in the case of Codex's `notify`),
  opens the local socket, dumps, and exits. No parsing beyond what routing
  requires, no network I/O, no model loading.
- **`hookd`** — a long-lived Rust daemon. It holds all the logic: normalization,
  policy, queue, STT, the TTS bridge, and IPC for the CLI and the GUI.

Rust for the three reasons that decide it here: a static binary with no runtime
to ship on all three systems, millisecond startup, and mature bindings for what
this project needs in audio, PTY, and ONNX.

## Consequences

**Good.** The hook's cost stays near the irreducible minimum. A dead daemon or a
missing socket becomes a silent exit with code 0 — the agent does not stall. The
logic lives in one place, and the CLI, the GUI, and the hooks are equal clients.

**Bad.** Two pieces to install, version, and keep compatible; the IPC protocol
becomes a real contract, with a version. And there is a daemon running in the
background, with everything that implies for lifecycle and autostart on all
three systems.

**Constrains.** Nothing that needs state between events may live in `hookc`.
Every temptation to add "just a small cache in the client" violates this
decision.

## Alternatives

**One binary; the hook does everything.** Rejected: loading an STT model on every
`PreToolUse` is unworkable, and there is no way to cut off a previous utterance
without state.

**Core in Python, alongside `voice-clone`.** Rejected on startup latency, though
it would have been faster to build. A thin shell client plus a socket would
mitigate it, but the GUI and Windows packaging would still be worse.

**Go.** Comparable startup and distribution; a poorer audio and ASR ecosystem,
and whisper.cpp would need CGo anyway — losing the advantage.

## Revision — 2026-08-23

The premise was measured after the decision and held: `cvb-hook` in release costs
**1.94 ms per invocation**, including the shell `fork` and the pipe, with the
daemon up. Numbers and method in [`memory/measurements.md`](../../../memory/measurements.md).

The decision does not change; the record exists so nobody has to re-measure, and
so a future regression has a baseline.
