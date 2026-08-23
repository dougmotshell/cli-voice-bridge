> Translation of [`measurements.md`](measurements.md), the source of truth.

# Measurements

Numbers measured on this machine. They exist so nothing that has been measured
gets re-decided by intuition — and so a change that made something worse is
noticeable.

## Hook client latency — 2026-08-23

The point of ADR-0001 is that `hookc` runs in series with the AI agent, hundreds
of times per session, and therefore has to be nearly free. Now it is measured:

| Measure | Value |
|---|---|
| 200 invocations of `cvb-hook` (release), daemon up | 387 ms |
| **Average per call** | **1.94 ms** — and that includes the shell `fork` and the pipe |
| `cvb-hook` with no daemon (failure path) | exits 0, silent |

Reference point: Python startup with `voice-clone`'s imports lands in the tens of
milliseconds, before any useful work. The order of magnitude is what ADR-0001
predicted.

**If this ever goes up**, the suspect is a new dependency in `hookc`. The crate
deliberately has no `clap`: for three positional arguments, its startup cost does
not pay for itself.

## Binary sizes (release, Linux x86_64) — 2026-08-23

Profile `opt-level = "z"`, `lto`, `strip`, `panic = "abort"`.

| Binary | Size |
|---|---|
| `cvb-hook` | 340 KB |
| `cvb-hookd` | 424 KB |
| `cvb` | 612 KB (the only one with `clap`) |
| `cvb-ptywrap` | 288 KB (still just the not-implemented message) |

## Not measured yet

- STT latency and error rate in pt-BR — that is what closes
  [ADR-0006](../docs/en-US/decisions/0006-stt-offline-na-maquina.md), and the
  engine decision comes from there, not from a comparison table.
- Time between an event arriving and the first sound coming out. That is the
  number the person feels; it does not exist yet because synthesis is not wired.
