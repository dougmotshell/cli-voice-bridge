# ADR-0007 — A canonical schema of "moments" between the CLIs and the voice

> Translation of [`../../pt-BR/decisions/0007-esquema-canonico-de-momentos.md`](../../pt-BR/decisions/0007-esquema-canonico-de-momentos.md),
> which is the source of truth.

**Status:** accepted — 2026-08-23
**C4 level:** [component](../architecture/03-component.md)
**Specs this decision moves:** [event-normalization](../specs/event-normalization.md), [speech-output](../specs/speech-output.md), [configuration](../specs/configuration.md)

## Context

The three CLIs report similar things under different names and formats: Claude
uses `snake_case` in the payload and `PascalCase` in the event name; Copilot uses
`camelCase` and accepts two event spellings; Codex's `notify` uses `kebab-case`.
The event sets do not coincide, and each vendor changes its own whenever it likes.

## Decision

There is a closed vocabulary of **moments** — `decision.needed`,
`turn.finished`, `subagent.started`, and so on — and it is the only thing that
crosses the boundary between the adapters and the rest of the system. Each
adapter translates its CLI's dialect; policy, queue, GUI, and configuration know
only moments.

An unknown event is not discarded: it becomes `error` carrying the raw name, and
`cvb doctor` complains. That is how a silent rename by a vendor surfaces instead
of disappearing.

## Consequences

**Good.** The speech policy is written once. Configuration speaks in moments,
which is what the person understands — "when a decision is needed, always speak"
holds across all three CLIs. Adding a fourth CLI means writing an adapter, not
touching the core.

**Bad.** One translation layer to maintain, and CLI-specific detail that does not
fit the vocabulary sits underused in the `detalhe` field. A moment that exists in
only one CLI (`MessageDisplay`, in Claude) produces asymmetric behavior that must
be documented, not hidden.

**Constrains.** No consumer may read the raw payload to make a decision. If it
needed to, either a moment is missing from the vocabulary or a field is missing
from the schema — both are fixed by extending the schema, not by piercing the
boundary.

## Alternatives

**Each adapter talking straight to TTS.** Rejected: it triples the policy, and
the divergence falls off the radar until it breaks.

**Passing the raw payload through.** Rejected: it forces the GUI and the
configuration to know all three dialects, and ties the interface to a
third-party format.

**Adopting one of the three vocabularies as canonical** (Claude's, the richest).
Rejected: it locks the project to one vendor's naming choices and gets awkward
when they rename something.
