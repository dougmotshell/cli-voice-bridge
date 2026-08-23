> Translation of [`MEMORY.md`](MEMORY.md), the source of truth.

# Project memory

Index. One line per entry, detail in the topic file. Keep it under 200 lines.
What goes here is what **cannot** be derived from the code or the git history.

- [Event surfaces of the AI CLIs](event-surfaces.en-US.md) — what each CLI
  exposes, which version it was verified on, and what has not been verified
- [Constraints inherited from voice-clone](voice-clone-constraints.en-US.md) —
  what the neighbouring project already learned and is not worth rediscovering
- [Measurements](measurements.en-US.md) — hook client latency and binary sizes,
  with the date
- [Known gaps](lacunas-conhecidas.en-US.md) — what does not work yet, why, and
  where it is described

TODO: add entries as the project moves. Obvious candidates: STT latency on the
target machine, and what broke in which CLI update.
