# ADR-0006 — Speech recognition offline, on the machine

> Translation of [`../../pt-BR/decisions/0006-stt-offline-na-maquina.md`](../../pt-BR/decisions/0006-stt-offline-na-maquina.md),
> which is the source of truth.

**Status:** accepted in principle; **engine open** — 2026-08-23
**C4 level:** [component](../architecture/03-component.md)
**Specs this decision moves:** [speech-input](../specs/speech-input.md)

## Context

Answering by voice requires transcription. What one dictates to a coding agent is
the work itself: repository names, paths, snippets of code, sometimes a client
name. Sending that to a transcription service is sending the work outside.

`voice-clone` already established the principle on the synthesis side: no audio
leaves the machine, and that is the central requirement, not a preference. It
would make no sense to honor it on output and violate it on input.

## Decision

STT runs **on the machine, offline**, supporting pt-BR and en-US. No microphone
audio and no transcription leaves the machine. Audio is discarded right after
transcription.

**The engine stays open.** The candidates and what separates them:

| Engine | For | Against |
|---|---|---|
| `whisper.cpp` | genuinely multilingual, high quality, mature Rust binding, runs well on CPU | larger model, higher latency, streaming is an approximation |
| `sherpa-onnx` | lighter, real streaming, several model families | pt-BR quality varies a lot by model |
| Vosk | light, mature, offline by design | quality below Whisper in pt-BR |
| Moonshine | very low CPU latency | English only — disqualifying for the main use |

The decision comes from **measurement on the target machine**, not from a
comparison table. That is exactly how `voice-clone` picked XTTS-v2, and it was by
measuring that it found threads should match physical cores, not logical ones.

## Consequences

**Good.** The work does not leak. It works without a network. No per-minute cost.

**Bad.** Model weights to download and version outside git. Quality below a cloud
service, especially in pt-BR with technical terms. CPU cost competing with XTTS,
which is also CPU-bound.

**Constrains.** No cloud STT provider is introduced without an ADR superseding
this one. It is because of this constraint that the closed-answer mode
([speech-input](../specs/speech-input.md)) uses a restricted vocabulary: with a
smaller local model, a closed vocabulary is what makes the decision trustworthy.

## Alternatives

**Cloud STT.** Rejected for the reason above.

**No voice input, output only.** Rejected: the person explicitly asked for every
form of answering.

**The system's native recognition** (macOS Dictation, Windows Speech
Recognition). Rejected: uneven quality, uneven availability, and on macOS
advanced dictation may involve an Apple server.

## Revision

TODO: close the engine. It comes out of a test with real pt-BR audio, measuring
latency and error rate in the two situations that matter: closed vocabulary
("yes", "no", "cancel") and dictating a technical prompt. Once decided, write
ADR-00NN superseding this one.
