> Translation of [`voice-clone-constraints.md`](voice-clone-constraints.md), the
> source of truth.

# What voice-clone already learned

`~/www/voice-clone` is a read-only external dependency
([ADR-0003](../docs/en-US/decisions/0003-tts-delegado-ao-voice-clone.md)). These
lessons cost time over there and are not worth rediscovering here.

**Loading XTTS-v2 takes about 30 seconds.** That is why the sidecar is a
long-lived process and not a `spawn` per phrase. Any proposal that reopens the
model per utterance is wrong by construction.

**Threads = physical cores, not logical ones.** With hyperthreading on, the same
sentence took 2.5× longer. Counterintuitive and measured. Do not "optimize" for
`os.cpu_count()`.

**PyTorch needs the CPU index on Linux**, otherwise it drags in ~2.5 GB of
useless CUDA on a machine with no GPU. If the sidecar ever gets its own
environment, inherit that pin.

**The XTTS-v2 license is CPML: it forbids commercial use.** This project inherits
the ceiling for as long as it depends on it. Acceptable — it is personal — but do
not suggest commercial use.

**Voice audio is biometric data.** `vozes/` and `saida/` are gitignored over
there, and no audio leaves the machine. That is its central requirement, and it
holds equally here, including microphone recordings and transcriptions.

**Windows breaks on encoding.** With output redirected, the default encoding is
the locale's and accents blow up with `UnicodeEncodeError`. `falar.py` solves it
with `reconfigure(encoding="utf-8")` on `stdout` and `stderr`. Inherit the lesson
in the sidecar instead of rediscovering it.

**Its official diagnostic is `falar.py checar`.** Before suspecting
`cli-voice-bridge`, run that.
