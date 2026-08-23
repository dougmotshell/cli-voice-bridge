> Translation of [`lacunas-conhecidas.md`](lacunas-conhecidas.md), the source of
> truth.

# Known gaps

What does **not** work, as of 2026-08-23. It exists so nobody discovers by trial
and error what is already known, and so none of this gets mistaken for a new
defect.

Each one has a document explaining why and what is left to decide; here there is
only the summary and the pointer.

| Gap | Symptom for the user | Where it is described |
|---|---|---|
| **Presence detection** | `falar = "ausente"` stays silent instead of speaking | [presence-detection](../docs/en-US/specs/presence-detection.md) |
| **Graceful shutdown** | A daemon killed by a signal leaves the socket on disk and may leave audio playing | [daemon-lifecycle](../docs/en-US/specs/daemon-lifecycle.md) |
| **Audio cache ceiling** | The `cache-audio/` directory only grows | [speech-output](../docs/en-US/specs/speech-output.md) |
| **Sidecar supervision** | The sidecar dies and nothing brings it back; the voice becomes the system one | [daemon-lifecycle](../docs/en-US/specs/daemon-lifecycle.md) |
| **Voice input** | No answering by speaking; no `cvb listen` | [speech-input](../docs/en-US/specs/speech-input.md) |
| **PTY wrapper** | No continuous narration and no text injection | [capture-transports](../docs/en-US/specs/capture-transports.md) |
| **GUI** | Does not exist | [interfaces](../docs/en-US/specs/interfaces.md) |
| **Long-message summary** | The assistant's message is cut by character count, not summarized | [speech-output](../docs/en-US/specs/speech-output.md) |
| **IPC on Windows** | The daemon does not start: named pipes not implemented | [portability](../docs/en-US/specs/portability.md) |

## What the pattern in them teaches

**Three of these gaps share a root:** presence, graceful shutdown, and sidecar
supervision are all things the operating system does better than the project.
Presence is idle time the OS already measures; shutdown is signal handling;
supervision is systemd or launchd. The temptation to hand-roll each one is what
should be resisted.

**Two are invisible until they hurt:** the uncapped cache and the log retention
that never prunes. Neither bothers anyone in the first month. That is why they
are written down.

**The Windows one is the only one that stops the project from running.** The
others degrade. If a Windows machine ever becomes available for testing, that is
the first one.
