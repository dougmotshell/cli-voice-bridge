# Decisões de arquitetura (ADR)

Formato MADR. Um arquivo por decisão, `NNNN-titulo-em-kebab.md`. Template em
[`templates/adr.md`](../../templates/adr.md).

**Append-only.** ADR aceito nunca é reescrito. Se a decisão muda, escreve-se um
ADR novo e o antigo passa a `Status: substituído por NNNN`. Número não se
reaproveita.

| # | Decisão | Status |
|---|---|---|
| [0001](0001-nucleo-em-rust-com-cliente-de-hook-separado.md) | Núcleo em Rust, com o cliente de hook separado do daemon | aceito |
| [0002](0002-gui-em-tauri-v2.md) | GUI em Tauri v2 | aceito |
| [0003](0003-tts-delegado-ao-voice-clone.md) | Síntese de voz delegada ao `voice-clone`, por sidecar | aceito |
| [0004](0004-hooks-oficiais-como-transporte-primario.md) | Hooks oficiais como transporte primário | aceito |
| [0005](0005-wrapper-pty-como-transporte-complementar.md) | Wrapper PTY como transporte complementar e opcional | aceito |
| [0006](0006-stt-offline-na-maquina.md) | Reconhecimento de fala offline, na máquina | aceito no princípio; motor em aberto |
| [0007](0007-esquema-canonico-de-momentos.md) | Um esquema canônico de "momentos" | aceito |
| [0008](0008-ipc-por-socket-local.md) | IPC por socket local, nunca por porta TCP | aceito |

TODO: decisões ainda não tomadas, que virarão ADR quando houver medição —
motor de STT (fecha o 0006), resumidor de mensagem longa, framework de
front-end da GUI, e forma de distribuição.
