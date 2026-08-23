# C4 level 3 — Components of `hookd`

> Translation of [`../../pt-BR/architecture/03-component.md`](../../pt-BR/architecture/03-component.md),
> which is the source of truth.

What lives inside the daemon. The surrounding pieces are in
[level 2](02-container.md).

```mermaid
graph TB
    subgraph entrada["Input"]
        ipc["ipc::server<br/>accepts hookc, ptywrap,<br/>cvb and the GUI"]
        adapters["adapters::{claude, codex, copilot}<br/>one per CLI, knows its dialect"]
    end

    normalize["core::normalize<br/>payload → canonical moment<br/>+ deduplication"]
    policy["policy<br/>speak? how urgent?<br/>presence, profile, project"]
    redact["redact<br/>strips secrets, shortens paths"]
    template["template<br/>moment → phrase"]
    summarize["summarize<br/>shortens a long message"]
    queue["speech::queue<br/>priority, collapsing,<br/>cutting, expiry"]
    tts["speech::tts<br/>bridge to the sidecar<br/>+ cache + OS fallback"]
    play["audio::playback"]

    subgraph escuta["Listening"]
        hotkey["listen::hotkey<br/>global shortcut"]
        capture["audio::capture"]
        vad["listen::vad"]
        stt["listen::stt<br/>engine TODO"]
        resolve["listen::resolve<br/>closed vocabulary<br/>or dictation"]
    end

    reply["reply<br/>delivers back to the CLI: hook response,<br/>PTY stdin, clipboard, or protocol"]
    state["state<br/>live sessions, log, metrics"]
    config["core::config<br/>layers and hot reload"]

    ipc --> adapters --> normalize --> policy
    policy -->|"worth speaking"| redact --> template --> summarize --> queue --> tts --> play
    policy -->|"needs an answer"| hotkey
    hotkey --> capture --> vad --> stt --> resolve --> reply
    normalize --> state
    config -.->|"reads"| policy
    config -.-> template
    config -.-> tts
    config -.-> resolve
    queue -.->|"cuts the speech in progress"| play
```

## Components

| Component | Responsibility | Spec |
|---|---|---|
| `ipc::server` | Accept connections from local clients | [interfaces](../specs/interfaces.md) |
| `adapters::*` | Translate each CLI's dialect. One module per CLI | [event-normalization](../specs/event-normalization.md) |
| `core::normalize` | Canonical moment and deduplication across transports | [event-normalization](../specs/event-normalization.md) |
| `policy` | Decide whether to speak, and how urgently, given presence and profile | [speech-output](../specs/speech-output.md) |
| `speech::redact` | Strip secrets before anything else | [speech-output](../specs/speech-output.md) |
| `speech::template` | Turn a moment into a short pt-BR phrase | [speech-output](../specs/speech-output.md) |
| `speech::Voz` | Orchestrates redact → synthesize → play, with cache and degradation | [speech-output](../specs/speech-output.md) |
| `core::sidecar` | Client of the synthesis sidecar | [speech-output](../specs/speech-output.md) |
| `core::audio` | Playback through a system program, and the emergency voice | [ADR-0009](../decisions/0009-reproducao-por-reprodutor-do-sistema.md) |
| `speech::queue` | Priority, collapsing of repeats, cutting, expiry — **does not exist yet** | [speech-output](../specs/speech-output.md) |
| `listen::*` | Shortcut, capture, VAD, transcription, resolution | [speech-input](../specs/speech-input.md) |
| `reply` | Deliver the answer through the right path for each CLI | [speech-input](../specs/speech-input.md) |
| `core::config` | Configuration layers and hot reload | [configuration](../specs/configuration.md) |
| `state` | Live sessions, log with retention, metrics for the GUI | — |

## Dependencies between components

`adapters` depends on `core`; `core` does not depend on `adapters`. That is what
lets a fourth CLI be added without touching the core
([ADR-0007](../decisions/0007-esquema-canonico-de-momentos.md)).

`redact` runs **before** `template` and before any logging. A secret must not
reach the template or the disk.

`core::audio` and `core::sidecar` sit in the core, not in `hookd`, because
`cvb doctor` has to check them with no daemon running. The rule that emerged:
**`core` is mechanism, `hookd` is policy.**

Today `Voz::falar` is synchronous and serialized behind a mutex — with no queue,
two threads speaking at once would produce overlapping audio. The queue with
priority, collapsing, and cutting is still to be built.

## Level 4 (code)

Deliberately absent. There is little code yet, and even later a class diagram
would not pay for its maintenance in a project this size. If some module gets
intricate enough to justify one, create `04-code.md` for that module alone — not
for the whole system.
