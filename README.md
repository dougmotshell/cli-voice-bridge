# cli-voice-bridge

Dá voz aos CLIs de IA. Fala em voz alta os momentos em que Claude Code, Codex CLI
e GitHub Copilot CLI tentam interagir com você — pedido de permissão, pergunta
pendente, turno concluído, subagente iniciado — e aceita a sua resposta falada.

Roda em Linux, macOS e Windows, em qualquer terminal, inclusive os integrados do
VS Code e do IntelliJ. Tudo local: nenhum áudio, transcrição ou texto de trabalho
sai da máquina. Projeto pessoal, sem uso comercial.

> **Estado: esqueleto que roda.** Compila sem avisos, 21 testes passam, e o
> caminho evento → momento foi exercitado ponta a ponta com payloads reais dos
> três CLIs. Fala, escuta, fila, configuração e instalador de hooks ainda não
> existem — tudo marcado `TODO:` nos documentos.

## Documentação

| Documento | Conteúdo |
|---|---|
| **[AGENTS.md](AGENTS.md)** | Contrato canônico: stack, armadilhas, convenções |
| **[Índice da documentação](docs/README.md)** | As quatro árvores e o que há em cada uma |
| **[Manual](docs/pt-BR/manual/README.md)** | Instalar, configurar, usar, resolver problemas |
| **[Arquitetura C4](docs/pt-BR/architecture/README.md)** | Contexto, contêineres, componentes |
| **[Specs](docs/README.md#specs)** | Uma capacidade por arquivo |
| **[ADRs](docs/pt-BR/decisions/README.md)** | As oito decisões e seus porquês |

## Como funciona

Os três CLIs disparam hooks — comandos que o próprio processo do CLI executa
quando algo acontece. É por isso que o projeto é indiferente ao terminal: quem
executa o hook é o CLI, dê no mesmo se ele abriu no Windows Terminal, no painel
do IntelliJ ou numa sessão remota.

O hook é atendido por um binário minúsculo que só repassa o evento a um daemon.
O daemon traduz os três dialetos num vocabulário único de **momentos**, decide o
que merece ser falado, e manda sintetizar na sua voz clonada — pelo
[`voice-clone`](../voice-clone), que roda XTTS-v2 offline em CPU. A resposta
volta por voz: pergunta fechada resolvida pelo próprio hook, ou ditado
transcrito localmente.

Detalhes em [docs/pt-BR/architecture/01-context.md](docs/pt-BR/architecture/01-context.md).

## Stack

Núcleo e CLI em **Rust** (o hook precisa custar quase nada), GUI em **Tauri v2**,
síntese delegada ao `voice-clone` por um sidecar **Python** que mantém o modelo
carregado. Os porquês estão nos [ADRs](docs/pt-BR/decisions/README.md).

## Dependências

- [`voice-clone`](../voice-clone) — motor de síntese, tratado como dependência
  externa somente leitura. Precisa estar funcionando (`falar.py checar`).
- Pelo menos um dos CLIs: Claude Code, Codex CLI ou GitHub Copilot CLI.

## Compilar

```bash
cargo build --release
cargo test
cargo clippy --workspace --all-targets -- -D warnings
```

Rust estável (verificado com 1.98.0), edição 2021. Sem Rust na máquina:
`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`

## Experimentar agora

```bash
export CVB_SOCKET=/tmp/cvb.sock
./target/release/cvb-hookd &                    # o daemon imprime os momentos

echo '{"session_id":"s1","hook_event_name":"PermissionRequest","tool_name":"Bash"}' \
  | ./target/release/cvb-hook --origem claude --transporte hook
# → decision.needed [Claude/hook] quer usar Bash

./target/release/cvb doctor
./target/release/cvb daemon status
```

## Próximo passo

Implementar o instalador de hooks (`cvb install`) e a ponte com o sidecar de
síntese. Ver `AGENTS.md`, seção *Comandos*.
