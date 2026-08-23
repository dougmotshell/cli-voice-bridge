# Spec — Presence detection

> Translation of [`../../pt-BR/specs/presence-detection.md`](../../pt-BR/specs/presence-detection.md),
> which is the source of truth.

**Capability:** know whether the person is nearby and paying attention, so that
speaking only happens when it adds something.

**ADRs constraining this spec:** [ADR-0008](../decisions/0008-ipc-por-socket-local.md).
**C4 level:** [component](../architecture/03-component.md) — the `policy::presenca` module.

## State

**Not implemented.** It is the only part of the speech policy that does not hold
today: `falar = "ausente"` stays silent instead of speaking, which is the
fallback documented in [speech-output](speech-output.md) ("assume present, speak
less"). Until this spec becomes code, anyone who wants to hear a medium-urgency
moment has to mark it explicitly as `"sempre"`.

## Problem

`falar = "ausente"` is the setting that cuts the most noise: it speaks only when
the person is not looking. If they have the terminal in front of them and just
typed, they already saw the permission request on screen — hearing the same thing
out loud is redundancy, and redundancy is what makes someone turn the project off.

The problem is that "not looking" has two readings with very different costs, and
picking the wrong one blocks the feature on an entire platform.

## Scope

In: deciding present/away and handing that to the speech policy.
Out: what to do with the answer — that is [speech-output](speech-output.md). Also
out: detecting that the person came back **after** an utterance started; that
already exists and arrives as an event (`user.returned`), not from a sensor.

## The two readings of "away"

| Reading | What it measures | Cost |
|---|---|---|
| **Idle at the machine** | Time since the last keystroke or mouse movement, anywhere | Low and uniform across all three systems |
| **Looking at that window** | Which window has focus, and whether it is that CLI session's terminal | High, and impossible on Wayland without compositor cooperation |

**The recommendation is the first.** Not because the second is worse — it is
better — but because the second runs into a problem no amount of implementation
effort solves: **the daemon does not know which window that CLI is in.** The hook
payload carries `session_id`, `cwd`, and `permission_mode`; it does not carry a
window identifier, and there is no reliable way to correlate a CLI process with a
terminal window, let alone with a tab in an IDE's integrated terminal.

In other words: even with window focus solved on Linux and macOS, the hard half
would still be missing. Idle time does not have that problem — it is a question
about the person, not about window topology.

## Design

```
policy::presenca::estado() -> Presenca { Presente, Ausente, Desconhecida }
```

`Desconhecida` is a legitimate answer and not an error: on Wayland without a
portal, it is what there is. Consumers treat `Desconhecida` as `Presente` —
assuming present makes it speak less, and silence bothers people less than noise.

Default threshold: **60 seconds** without input. TODO: measure; 60 s is an
informed guess, not a measurement.

### How to get idle time on each system

| System | Path | Note |
|---|---|---|
| Linux/X11 | `XScreenSaver` extension (`XScreenSaverQueryInfo` → `idle`) | Reliable and cheap |
| Linux/Wayland | `org.freedesktop.ScreenSaver` over D-Bus, or the `ext-idle-notify-v1` protocol | Depends on the compositor; GNOME and KDE expose it, others do not |
| macOS | `CGEventSourceSecondsSinceLastEventType` with `kCGAnyInputEventType` | No special permission |
| Windows | `GetLastInputInfo` | No special permission |

None of these require Accessibility permission and none record content — only the
number of seconds since the last input event. See *Privacy*.

TODO: decide whether to take a native dependency per system, or have the daemon
delegate to an external command the way
[ADR-0009](../decisions/0009-reproducao-por-reprodutor-do-sistema.md) did for
playback. The analogy is tempting, but here the query is frequent and one process
per query would not pay for itself.

## Data and contracts

The state is queried at the moment of deciding to speak, not subscribed to.
Nothing is stored: no idle-time history, no timestamp of when the person was last
present.

## Privacy

Idle time is a number of seconds. It does **not** read which key was pressed,
which window has focus, which application is open, or the window title — even
where the system would allow it. That information is not needed for the decision
and, once read, would become one more thing to protect.

None of it leaves the machine, like the rest of the project.

## Alternatives considered

**Window focus instead of idle time.** A better signal, and rejected for the
reason above: the daemon does not know which window belongs to that CLI, so the
better signal would not be applicable anyway.

**Asking the CLI itself.** None of the three exposes "the person is looking at
me". What exists is `Notification` with `idle_prompt` in Claude, which says the
*agent* is idle — the opposite information.

**Letting the person toggle it by hand** (`cvb away` / `cvb back`). Rejected as
the solution: nobody remembers to announce that they left, and the value is
precisely in not having to. It stays useful as a complement — TODO: evaluate
alongside profiles.

**Not implementing it and leaving `ausente` as a synonym for `nunca`.** That is
what holds today, and it is honest while it is written down. It does not work as
a final decision: it removes half the usefulness of the configuration.

## Test plan

TODO: write it. At minimum: an injectable idle-clock abstraction, so the policy
can be tested without depending on the system; and a test asserting that
`Desconhecida` behaves as `Presente`. The per-system part cannot be tested in CI
without a graphical session — it goes on the manual checklist in the manual.

## Open questions

- The idle threshold (above).
- Native dependency per system or external command (above).
- What to do when the person is present but on another monitor, with their back
  to the terminal. Likely answer: nothing — that is the limit of what idle time
  measures, and accepting the limit beats faking a precision that does not exist.
