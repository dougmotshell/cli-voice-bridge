# Spec — Portabilidade

**Capacidade:** funcionar de verdade em Linux, macOS e Windows.

**ADRs que restringem este spec:** [ADR-0001](../decisions/0001-nucleo-em-rust-com-cliente-de-hook-separado.md),
[ADR-0002](../decisions/0002-gui-em-tauri-v2.md), [ADR-0008](../decisions/0008-ipc-por-socket-local.md).
**Nível C4:** [contêiner](../architecture/02-container.md).

## Problema

Quatro coisas deste projeto divergem nos três sistemas, e nenhuma delas é
detalhe: IPC, pseudo-terminal, atalho global e áudio. Tratar como "depois a gente
adapta" produz um projeto que só roda numa máquina.

## Matriz

| Assunto | Linux | macOS | Windows |
|---|---|---|---|
| IPC daemon ↔ clientes | socket UNIX em `$XDG_RUNTIME_DIR` | socket UNIX | named pipe |
| Pseudo-terminal | `openpty` | `openpty` | ConPTY (Windows 10 1809+) |
| Atalho global | X11 direto; Wayland exige o portal `GlobalShortcuts` | precisa de permissão de Acessibilidade, concedida pela pessoa | `RegisterHotKey` |
| Captura de áudio | ALSA/PipeWire | CoreAudio | WASAPI |
| Reprodução | idem | idem | idem |
| Voz de fallback | `espeak-ng` | `say` | SAPI |
| Configuração | `~/.config/cli-voice-bridge/` | `~/Library/Application Support/` | `%APPDATA%` |
| Autostart | unidade systemd de usuário | `launchd` LaunchAgent | Tarefa Agendada ou chave Run |
| Comando de hook | `sh -c` | `sh -c` | `cmd`/`pwsh` — o Copilot tem campos `bash` e `powershell` separados |

## Regras

**Nada de caminho fixo.** Todo caminho de configuração, socket, cache e log sai
de uma função que conhece os três sistemas. Um literal `~/.config` no código é
defeito.

**Wayland é o caso difícil.** Atalho global e simulação de teclado são
restringidos de propósito. O caminho suportado é o portal; onde ele não existe, o
atalho global não funciona e o `cvb doctor` **diz isso** em vez de falhar em
silêncio. O modo de ditado por clipboard continua funcionando, e é por isso que
ele é o padrão.

**Degradar com voz, não com silêncio.** Recurso indisponível na plataforma vira
aviso explícito no `doctor` e na GUI, com o que a pessoa pode fazer no lugar.

**Windows não é porte de segunda.** ConPTY e named pipes desde o começo, não
depois. O `voice-clone` já resolveu a parte de encoding (`reconfigure(utf-8)` em
`stdout`/`stderr`) — herdar a lição em vez de redescobrir.

## Distribuição

TODO: decidir. As opções na mesa: binários por plataforma anexados a uma release,
`cargo install` para quem tem Rust, e pacote da GUI pelo `tauri build`
(`.AppImage`/`.deb`, `.dmg`, `.msi`). Projeto pessoal: começar por binários numa
release e por instruções de compilar, sem assinatura de código.

Assinatura e notarização (macOS) e SmartScreen (Windows) vão incomodar. Aceitável
para uso pessoal; documentar o contorno no manual em vez de fingir que não
existe.

## Plano de teste

TODO: escrever. Sem os três sistemas à mão, o realista é: CI com os três
runners rodando `cargo test` e `cvb doctor --offline`, mais uma lista de
verificação manual no manual para o que exige áudio e permissão de verdade.

## Questões em aberto

- Distribuição (acima).
- Se vale suportar Wayland sem portal com alguma alternativa, ou declarar sem
  suporte e pronto.
