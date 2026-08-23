# Documentation (en-US)

> Translation of [`../pt-BR/README.md`](../pt-BR/README.md), which is the source
> of truth. Change the pt-BR file first, then bring the translation along.

| Tree | Standard | One file per | Index |
|---|---|---|---|
| [`architecture/`](architecture/) | C4 | level | [README](architecture/README.md) |
| [`specs/`](specs/) | SDD | capability | below |
| [`decisions/`](decisions/) | ADR (MADR) | decision | [README](decisions/README.md) |
| [`manual/`](manual/) | user manual | user task | [README](manual/README.md) |

## Specs

| Spec | Capability |
|---|---|
| [event-normalization](specs/event-normalization.md) | Translate the events of all three CLIs into a single vocabulary of "moments". **This is the source of truth for the event map** |
| [capture-transports](specs/capture-transports.md) | Receive events from any terminal: hooks, `notify`, PTY, `stream-json`, ACP |
| [speech-output](specs/speech-output.md) | Turn moments into speech without becoming noise |
| [speech-input](specs/speech-input.md) | Answer by voice, in all four ways |
| [configuration](specs/configuration.md) | Layers, precedence, and the set of keys |
| [interfaces](specs/interfaces.md) | CLI and GUI, with real parity |
| [portability](specs/portability.md) | Linux, macOS, and Windows for real |

The rules that hold across all four trees live in [`../README.md`](../README.md).
