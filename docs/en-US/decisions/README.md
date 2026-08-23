# Architecture decisions (ADR)

> Translation of [`../../pt-BR/decisions/README.md`](../../pt-BR/decisions/README.md),
> which is the source of truth. Filenames match across both trees on purpose, so
> a translation is always a sibling and never a fork.

MADR format. One file per decision, `NNNN-kebab-title.md`. Template in
[`templates/adr.md`](../../../templates/adr.md).

**Append-only.** An accepted ADR is never rewritten. If the decision changes,
write a new ADR and mark the old one `Status: superseded by NNNN`. Numbers are
never reused.

| # | Decision | Status |
|---|---|---|
| [0001](0001-nucleo-em-rust-com-cliente-de-hook-separado.md) | Rust core, with the hook client separate from the daemon | accepted |
| [0002](0002-gui-em-tauri-v2.md) | GUI in Tauri v2 | accepted |
| [0003](0003-tts-delegado-ao-voice-clone.md) | Speech synthesis delegated to `voice-clone`, via a sidecar | accepted |
| [0004](0004-hooks-oficiais-como-transporte-primario.md) | Official hooks as the primary transport | accepted |
| [0005](0005-wrapper-pty-como-transporte-complementar.md) | PTY wrapper as a complementary, optional transport | accepted |
| [0006](0006-stt-offline-na-maquina.md) | Speech recognition offline, on the machine | accepted in principle; engine open |
| [0007](0007-esquema-canonico-de-momentos.md) | A canonical schema of "moments" | accepted |
| [0008](0008-ipc-por-socket-local.md) | IPC over a local socket, never a TCP port | accepted |

TODO: decisions not yet taken, which become ADRs once measured — the STT engine
(closes 0006), the summarizer for long messages, the GUI front-end framework,
and the distribution format.
