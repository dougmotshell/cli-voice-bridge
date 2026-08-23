# Spec — Speech output

> Translation of [`../../pt-BR/specs/speech-output.md`](../../pt-BR/specs/speech-output.md),
> which is the source of truth.

**Capability:** turn moments into speech, in the cloned voice, without becoming
noise.

**ADRs constraining this spec:** [ADR-0003](../decisions/0003-tts-delegado-ao-voice-clone.md),
[ADR-0007](../decisions/0007-esquema-canonico-de-momentos.md),
[ADR-0009](../decisions/0009-reproducao-por-reprodutor-do-sistema.md).
**C4 level:** [component](../architecture/03-component.md) — `hookd::speech`.

## State

| Part | State |
|---|---|
| Redaction | **implemented** — `speech::redact`, with decoy tests |
| Templates and discreet mode | **implemented** — `speech::template` |
| Bridge to the sidecar | **implemented** — `core::sidecar` |
| Phrase cache | **implemented** — keyed by (voice, language, text) |
| Playback and degradation | **implemented** — `core::audio` ([ADR-0009](../decisions/0009-reproducao-por-reprodutor-do-sistema.md)) |
| Policy by urgency | **partial** — `sempre` and `nunca` hold; `ausente` stays silent, for lack of presence detection |
| Queue with priority, collapsing, expiry, and cutting | **implemented** — `speech::queue` |
| Summary of a long message | **does not exist** — today just a character-count cut |
| Presence detection | **does not exist** — it is what `ausente` is waiting on |

## Problem

One agent turn generates dozens of moments. Speaking all of them is unbearable;
speaking too few defeats the purpose. And XTTS-v2 on CPU is not instantaneous —
synthesis takes a time comparable to the audio's duration, so the queue matters.

## Scope

In: the policy of what to say, queue and priority, cutting off speech in progress
(*barge-in*), redaction of sensitive content, the bridge to `voice-clone`, cache.
Out: how the moment arrived ([capture-transports](capture-transports.md)) and the
person's answer ([speech-input](speech-input.md)).

## Policy: what becomes speech

Defaults by urgency, all overridable per CLI, per project, and per moment
([configuration](configuration.md)):

| Urgency | Moments | Default behavior |
|---|---|---|
| critical | `decision.needed`, `input.needed` | always speaks, interrupts whatever is speaking |
| high | `turn.finished`, `turn.failed`, `error` | speaks, goes to the front of the queue |
| medium | `subagent.*`, `task.completed`, `tool.failed` | speaks only if the person is away or the session is in the background |
| low | `session.*`, `task.created`, `context.compacting` | does not speak; shows in the GUI |
| silent | `tool.started`, `tool.finished`, `message.text` | does not speak, unless narration mode is on |

**Presence.** "Away" is the criterion that avoids the worst of the noise: if the
terminal window has focus and there was a keystroke seconds ago, the person has
already seen it on screen and does not need to hear it. TODO: define focus
detection on all three systems, and what to do when it cannot be known (likely:
assume present, speak less).

**Narration mode** (opt-in): speaks `message.text` as the assistant writes. It
only makes sense with the `pty` or `stream-json` transport.

## Text: from moment to phrase

No moment is spoken raw. The chain is:

1. **Redaction.** Strips secrets before anything else — token, key, password, and
   whatever the configuration marks as sensitive. An absolute path becomes the
   file name. See *Privacy*.
2. **Template.** Each moment has a short pt-BR template, editable by the person:
   `decision.needed` → "Claude wants to run {tool}. Allow?";
   `turn.finished` → "{cli} finished. {summary}".
3. **Summary.** `last_assistant_message` usually has paragraphs. Speaking all of
   it is useless. Reduce to one or two sentences. TODO: decide the summarizer —
   local extractive (fast, no dependency) or a small local model. Calling a cloud
   model for this contradicts the principle of not leaking context.
4. **Hard limit.** No utterance exceeds N seconds (default TODO: start at 12).
   The rest stays in the GUI and comes out as "there's more on screen".

## Queue

A single queue, with priority and one player.

- Critical **cuts** what is playing and clears anything of lower urgency.
- High goes ahead of medium and low.
- Moments of the same session and same kind **collapse**: three consecutive
  `tool.failed` become "three tools failed".
- A moment older than the relevance window is discarded without speaking —
  announcing "I'm done" 40 seconds late is worse than silence.
- `UserPromptSubmit` (any CLI) arrives as `user.returned` and **cuts everything**:
  the utterance in progress and whatever is queued. If the person is typing, they
  are already back.
- `cvb mute` also cuts immediately, not just from the next utterance on.

**Cutting means killing the player process**, which is what the
[ADR-0009](../decisions/0009-reproducao-por-reprodutor-do-sistema.md) choice
allows. `Voz` keeps the running process precisely for that; whoever waits for the
end does so by polling, because blocking on the process would hold it against
whoever is trying to kill it.

## Bridge to voice-clone

`voice-clone` is a read-only external dependency, called through its CLI
contract, never by module import. The path comes from configuration.

```
<voice-clone venv>/bin/python falar.py falar <voice> "<text>" --saida <file> [--rapido]
```

The sidecar keeps the Python process **alive** between calls: loading XTTS-v2
takes about 30 seconds, and paying that per phrase is unacceptable. So the
sidecar is not a `spawn` per utterance — it is a long-lived local server with its
own queue, and `hookd` talks to it over a socket.

TODO: `falar.py` is a one-shot command today. Either the sidecar wraps
`vozclone.py` in a server loop (which touches `voice-clone`, requiring a separate
conversation), or the sidecar is written here and imports `vozclone` from that
venv. The second option respects "read-only" and is the working hypothesis.

**Degradation.** A dead sidecar or unavailable XTTS must not mute the system: it
falls back to the operating system voice (`espeak-ng`/`say`/SAPI) with a warning,
and `cvb doctor` says why. `cvb say` reports which path it spoke through —
"cloned voice" and "system voice" are very different results, and conflating them
would hide a dead sidecar. Speaking with an ugly voice beats not
saying that the agent is stuck waiting for permission.

**Cache.** Fixed phrases ("done", "I need permission") are few and repeat.
Synthesize once, store the WAV keyed by (voice, language, text). That turns most
utterances into instant playback. It lives in `<data>/cache-audio/`, named from a
non-cryptographic hash — it is a file name, not an integrity guarantee.

**The cache has no ceiling, and that is a real gap.** It grows with every new
phrase and nothing is ever removed. It does not hurt today because phrases repeat
a lot — the set of things the project says is small — but two things make it grow
without bound: summarized assistant messages (each one unique) and switching
voice or language, which invalidates everything without deleting anything.

What is left to decide, in order of importance:

1. **The ceiling.** By size on disk (say, 200 MB) or by file count. Size is what
   the person understands, and it is what shows up when it starts to hurt.
2. **Eviction policy.** Least-recently-used is the obvious candidate, and it
   requires keeping the last-access time — which `mtime` already gives for free,
   unless the filesystem is mounted `noatime`. Keeping an index of our own is
   more reliable and one more thing to keep in sync.
3. **When to prune.** At daemon startup is simplest; alongside graceful shutdown
   ([daemon-lifecycle](daemon-lifecycle.md)) is better timed.
4. **Invalidation by voice and language.** The key already includes both, so
   switching voice never returns the wrong audio — it just leaves the old files
   taking up space forever. Pruning by key prefix would solve it, and requires
   the key to stop being an opaque hash.

Until there is a decision, `cvb doctor` should at least **report the cache size**,
so the gap is visible before it becomes a problem. TODO: not even that exists.

## Privacy

Speaking is publishing into a shared environment. Whoever is in the room hears
the client name, the repository path, the code fragment.

- Redaction runs **before** the template, always, with no option to disable it
  for secrets.
- Discreet mode (`cvb quiet`) speaks only the category, never the content: "the
  agent needs a decision" instead of naming the command.
- No moment text is written to disk beyond the session log, which is gitignored
  and has a configurable retention.
- No audio, text, or transcription leaves the machine.

## Alternatives considered

**Speaking directly from the hook process.** Rejected: it blocks the agent for
the duration of synthesis and gives no way to cut the previous utterance.

**Cloud TTS (ElevenLabs, OpenAI).** Rejected: it sends the work's text outside,
costs money, and the cloned voice — the whole point of the project — already
exists locally.

**System voice only, no XTTS.** Rejected as the default, kept as the fallback.

## Test plan

TODO: write it. Minimum: a queue test (critical cuts medium), a collapsing test,
a redaction test with a battery of decoy secrets, and a manual smoke test
(`cvb say`) because only an ear judges naturalness.

## Open questions

- Which summarizer (above).
- Presence/focus detection on all three systems (above).
- What to do when several CLIs speak at once: different voices per tool, or a
  spoken prefix ("Codex:")? Different voices are nicer and more expensive to
  configure.
