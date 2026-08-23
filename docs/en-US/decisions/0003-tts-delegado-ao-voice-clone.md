# ADR-0003 — Speech synthesis delegated to voice-clone, via a sidecar

> Translation of [`../../pt-BR/decisions/0003-tts-delegado-ao-voice-clone.md`](../../pt-BR/decisions/0003-tts-delegado-ao-voice-clone.md),
> which is the source of truth.

**Status:** accepted — 2026-08-23
**C4 level:** [container](../architecture/02-container.md)
**Specs this decision moves:** [speech-output](../specs/speech-output.md)

## Context

The cloned voice already exists: `~/www/voice-clone` runs XTTS-v2 fully offline
on CPU, in pt-BR and en-US, and it is the reason this project exists. XTTS-v2 is
Python, and loading the model takes about 30 seconds.

Reimplementing it in Rust is off the table: the XTTS libraries are Python, and
the quality of the voice is the product.

## Decision

`voice-clone` is a **read-only external dependency**. A long-lived Python sidecar
loads the model once and serves `hookd` over a local socket. Integration happens
through `voice-clone`'s public contract, with the path coming from configuration
— never embedded in code, never by copying files.

There is always a degradation path: an unavailable sidecar falls back to the
operating system voice, with a warning in the GUI and in `cvb doctor`. Speaking
badly beats not warning that the agent is stuck waiting for permission.

## Consequences

**Good.** Zero duplication of the hard part. `voice-clone` evolves on its own.
Its privacy constraint — no audio leaves the machine — is inherited.

**Bad.** One more Python runtime to install and keep alive. The sidecar needs
supervision: if it dies, restart it. And the project inherits the XTTS-v2 CPML
license, which forbids commercial use — acceptable, because this project is
personal and non-commercial, but it is a real ceiling.

**Constrains.** Nothing here may `import vozclone` as if it were its own module,
nor edit `voice-clone`. Needing a change over there is a separate conversation,
with an ADR over there.

## Alternatives

**One `spawn` of `falar.py` per phrase.** Rejected: it would pay the ~30 s model
load on every utterance.

**Cloud TTS (ElevenLabs, OpenAI).** Rejected: it sends the work's text outside,
costs money, and throws away the cloned voice.

**OS voice only.** Rejected as the default — it is the fallback.

**Absorbing `voice-clone` into this repository.** Rejected: it duplicates
maintenance and kills the independent evolution of both.
