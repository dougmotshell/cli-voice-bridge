# ADR-0009 — Play audio through the system player, not through a library

> Translation of [`../../pt-BR/decisions/0009-reproducao-por-reprodutor-do-sistema.md`](../../pt-BR/decisions/0009-reproducao-por-reprodutor-do-sistema.md),
> which is the source of truth.

**Status:** accepted — 2026-08-23
**C4 level:** [component](../architecture/03-component.md)
**Specs this decision moves:** [speech-output](../specs/speech-output.md), [portability](../specs/portability.md)

## Context

The sidecar hands over a WAV file; something has to play it. The idiomatic answer
in Rust would be `rodio`, which pulls in `cpal` and a decoder.

But `cpal` on Linux compiles against ALSA: without `libasound2-dev` installed,
the build fails — and it fails in a project the person just cloned, before
anything works at all. On a development machine that is one `apt install` away;
for someone who just wants to run it, it is a wall at the door.

This project will eventually need audio capture for voice input, and a native
library will probably be unavoidable then. But playing a WAV is the simplest
possible case, and all three systems already ship a program that does exactly it.

## Decision

Play by invoking a system program, chosen on first use from a per-platform list
of candidates:

| System | Candidates, in order |
|---|---|
| Linux | `paplay`, `pw-play`, `aplay`, `ffplay` |
| macOS | `afplay` |
| Windows | `powershell` with `System.Media.SoundPlayer` |

The person can pin the command in `geral.reprodutor` in the configuration, and
then the list is not consulted. No player found is a failure **reported** by
`cvb doctor`, with the list of what was looked for — not a generic error.

## Consequences

**Good.** Zero build dependencies: `cargo build` works on a freshly cloned
machine, with no system package. An audio problem stays reproducible by hand —
the person runs the same command in a terminal and sees what happens. And the
system player already respects the user's output settings.

**Bad.** One process per utterance, with the startup cost that carries — small
next to the model's ~30 s, but not zero. There is no volume control, no fading,
no mixing. And **cutting off speech in progress** becomes killing a child
process, which is cruder than stopping an audio stream.

**Constrains.** While this decision holds, the speech queue cuts by interrupting
a process, not by pausing a stream. If cutting turns out to sound bad, that is
the signal to revisit — and probably together with audio capture, as a single
decision.

## Alternatives

**`rodio`/`cpal`.** Rejected for now: it requires `libasound2-dev` on Linux and
would turn the first `cargo build` into a hunt for a system package. It comes
back to the table when audio capture arrives, because the native dependency will
already have been paid for.

**Playing inside the Python sidecar.** Rejected: the sidecar exists to load XTTS
once, and giving it the audio-output role would conflate the two. Besides, the
fallback to the system voice has to play **without** the sidecar, precisely
because it may be dead.

**A dedicated audio server.** Rejected: complexity with no return for playing one
WAV at a time.
