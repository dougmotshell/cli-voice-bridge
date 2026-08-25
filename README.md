# cli-voice-bridge

Dá voz aos CLIs de IA. Fala em voz alta os momentos em que Claude Code, Codex CLI
e GitHub Copilot CLI tentam interagir com você — pedido de permissão, pergunta
pendente, turno concluído, subagente iniciado — e aceita a sua resposta falada.

Roda em Linux, macOS e Windows, em qualquer terminal, inclusive os integrados do
VS Code e do IntelliJ. Tudo local: nenhum áudio, transcrição ou texto de trabalho
sai da máquina. Projeto pessoal, sem uso comercial.

> **Estado: já fala, com fila, e se instala sozinho.** Compila sem avisos, 61
> testes passam, o caminho evento → momento → fila → voz foi exercitado ponta a
> ponta com payloads reais dos três CLIs, e `cvb install` liga os hooks sem
> apagar os alheios. Falta a entrada por voz.

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
[`voice-clone`](https://github.com/dougmotshell/voice-clone), que roda XTTS-v2 offline em CPU. A resposta
volta por voz: pergunta fechada resolvida pelo próprio hook, ou ditado
transcrito localmente.

Detalhes em [docs/pt-BR/architecture/01-context.md](docs/pt-BR/architecture/01-context.md).

## Stack

Núcleo e CLI em **Rust** (o hook precisa custar quase nada), GUI em **Tauri v2**,
síntese delegada ao `voice-clone` por um sidecar **Python** que mantém o modelo
carregado. Os porquês estão nos [ADRs](docs/pt-BR/decisions/README.md).

## Dependências

- [`voice-clone`](https://github.com/dougmotshell/voice-clone) — motor de
  síntese, tratado como dependência externa somente leitura. Instala-se com o
  `scripts/install.sh` dele (vai para `~/.local/share/voice-clone`) e precisa
  estar funcionando (`voice-clone checar`).
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
./target/release/cvb doctor          # diz o que falta antes de qualquer coisa

export CVB_SOCKET=/tmp/cvb.sock
./target/release/cvb-hookd &         # o daemon imprime os momentos e fala

echo '{"session_id":"s1","hook_event_name":"PermissionRequest","tool_name":"Bash"}' \
  | ./target/release/cvb-hook --origem claude --transporte hook
# → decision.needed [Claude/hook] quer usar Bash
# → e, se houver voz configurada: "Claude quer usar Bash. Autorizo?"

./target/release/cvb say "teste"     # falado — voz clonada | voz do sistema
./target/release/cvb voices
```

Sem o sidecar de síntese de pé, ele fala com a voz do sistema (`espeak-ng`,
`say`, SAPI) e diz que foi por aí — nunca fica mudo. Para a voz clonada:

```bash
CVB_VOICE_CLONE=~/.local/share/voice-clone \
  ~/.local/share/voice-clone/.venv/bin/python sidecar/servidor.py
```

## Ligar aos seus CLIs

```bash
cvb install --dry-run     # mostra o que mudaria; --diff para o diff completo
cvb install               # aplica
cvb uninstall             # tira só o que o cvb pôs
```

Ele **compõe** com os hooks que já existem: lê o arquivo, acrescenta as entradas
do `cvb`, preserva o resto, e guarda o original em `*.cvb-backup`. Se você já usa
outros hooks, eles continuam funcionando.

## Próximo passo

A entrada por voz: responder "sim" a um pedido de permissão sem tocar no teclado.
Ver `docs/pt-BR/specs/speech-input.md`.
