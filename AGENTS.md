# AGENTS.md — cli-voice-bridge

Contrato canônico deste projeto. Vale para qualquer CLI de IA; `CLAUDE.md` e
`.github/copilot-instructions.md` são adaptadores finos que importam este
arquivo. Edite aqui, não nos adaptadores.

**O que é:** uma ponte de voz entre a pessoa e os CLIs de IA (Claude Code,
Codex CLI, Copilot CLI). Fala em voz alta os momentos em que o agente tenta
interagir — pedido de permissão, tarefa concluída, subagente iniciado, pergunta
pendente — e aceita a resposta por voz. Uso pessoal, sem fim comercial.

**Estado: esqueleto que roda.** O workspace existe e o caminho evento → momento
está de pé e testado. Fala, escuta, fila, configuração e instalador de hooks
ainda não existem — estão marcados `TODO:` aqui e nos specs.

## Stack

| Camada | Escolha | Onde está decidido |
|---|---|---|
| Núcleo (daemon + CLI) | Rust | [ADR-0001](docs/pt-BR/decisions/0001-nucleo-em-rust-com-cliente-de-hook-separado.md) |
| GUI | Tauri v2 | [ADR-0002](docs/pt-BR/decisions/0002-gui-em-tauri-v2.md) |
| Síntese de voz (TTS) | delegada ao `voice-clone` por sidecar Python | [ADR-0003](docs/pt-BR/decisions/0003-tts-delegado-ao-voice-clone.md) |
| Reconhecimento (STT) | offline, na máquina | [ADR-0006](docs/pt-BR/decisions/0006-stt-offline-na-maquina.md) |
| Transporte de eventos | hooks oficiais + wrapper PTY + protocolo de agente | [ADR-0004](docs/pt-BR/decisions/0004-hooks-oficiais-como-transporte-primario.md), [ADR-0005](docs/pt-BR/decisions/0005-wrapper-pty-como-transporte-complementar.md) |

```
crates/core/      esquema de momentos, protocolo de IPC, caminhos por plataforma
crates/hookd/     o daemon: normaliza, decide, enfileira, fala e escuta
crates/hookc/     o cliente de hook (binário `cvb-hook`)
crates/cvb/       a CLI
crates/ptywrap/   o wrapper de PTY — declarado, ainda não implementado
gui/              a aplicação Tauri — ainda não criada, ver gui/README.md
sidecar/          a ponte Python para o voice-clone
```

## Comandos

```bash
cargo build --release              # o workspace inteiro
cargo test                         # testes
cargo clippy --all-targets -- -D warnings   # obrigatório antes de commit
cargo fmt --all

cvb doctor                         # diagnóstico — SEMPRE o primeiro passo
cvb daemon status
cvb say "texto"                    # TODO: depende do sidecar
cvb install --cli claude,codex     # TODO: instalador de hooks

# sidecar de síntese, com o interpretador do voice-clone
CVB_VOICE_CLONE=/caminho/voice-clone \
  /caminho/voice-clone/.venv/bin/python sidecar/servidor.py
```

`cvb doctor` é **sempre o primeiro passo** ao diagnosticar. Antes de suspeitar de
lógica, confira dispositivo de áudio, sidecar vivo e hooks realmente instalados.

Não há CI ainda. TODO: rodar `cargo test`, `clippy`, `fmt --check` e
`scripts/sync-ai-surfaces.py --check` nos três sistemas.

## Armadilhas — leia antes de mexer

**O cliente de hook não pode ser lento.** `PreToolUse` dispara centenas de vezes
por sessão e roda em série com o agente. O `hookc` precisa ser um binário que
abre socket, despeja o payload e sai — sem parsing pesado, sem I/O de rede, sem
carregar modelo. Toda a lógica mora no `hookd`. Um `hookc` de 40 ms é 40 ms de
lentidão em cada ferramenta que a pessoa vê ([ADR-0001](docs/pt-BR/decisions/0001-nucleo-em-rust-com-cliente-de-hook-separado.md)).

**Nunca sobrescreva a configuração de hooks de terceiros.** Esta máquina já roda
`rtk hook claude` em `PreToolUse` no Claude e no Codex. A instalação **compõe**:
lê o JSON existente, acrescenta a entrada do `cvb`, preserva o resto. Instalador
que reescreve `settings.json` inteiro apaga trabalho alheio.

**Hook que falha não pode travar o agente.** Falha de áudio, sidecar morto,
daemon fora do ar — tudo isso sai com código 0 e silêncio. A única exceção é a
decisão de permissão por voz, e mesmo ela cai para "perguntar na tela" quando o
reconhecimento não é confiável. Ver `docs/pt-BR/specs/speech-input.md`.

**Payloads de hook são de três dialetos diferentes.** Claude usa `snake_case`
(`tool_name`, `hook_event_name`); Copilot usa `camelCase` (`toolName`,
`sessionId`) e aceita o nome do evento em duas grafias; Codex usa `PascalCase` no
evento com payload `snake_case`. Não trate como um só formato — normalize na
borda, em `crates/core`. O mapa completo está em
`docs/pt-BR/specs/event-normalization.md`, e é a fonte da verdade.

**Áudio de voz é dado biométrico.** A voz clonada vem do `voice-clone`, cujos
diretórios `vozes/` e `saida/` já são segredo. Aqui vale o mesmo: nenhuma amostra
de voz, nenhuma gravação de microfone e nenhuma transcrição entra no git ou sai
da máquina. STT e TTS rodam localmente; não introduza provedor de nuvem sem ADR.

**O conteúdo falado vaza contexto.** O texto de um evento pode conter caminho de
arquivo, nome de cliente, trecho de código. Falar em voz alta é publicar num
ambiente compartilhado. A política de redação (`docs/pt-BR/specs/speech-output.md`) não
é enfeite.

**Licença do XTTS-v2 proíbe uso comercial.** O `voice-clone` usa CPML. Este
projeto herda a restrição enquanto depender dele. Não sugira uso comercial.

## Dependência do voice-clone

`~/www/voice-clone` é o motor de fala, tratado como
**dependência externa somente leitura**. A integração é pelo contrato de CLI dele
(`falar.py falar <voz> "texto"`), nunca por import de módulo, e o caminho vem da
configuração — nunca embutido no código. Mudança que exigiria alterar o
`voice-clone` é uma conversa separada, não um patch de lado.

## Convenções

**Idioma: pt-BR e en-US, sempre as duas.** pt-BR é a fonte da verdade e vem
primeiro; en-US é irmão de mesmo nome, abrindo com um ponteiro para o original.
Identificadores, nomes de arquivo e de branch em en-US. Acentuação completa.

Prosa nova nasce em pt-BR e só está pronta quando o irmão en-US existe. Vale para
o contrato, os adaptadores, as skills, as regras e as quatro árvores de `docs/`.

**Documentação mora em `docs/<idioma>/`, em uma das quatro árvores.** Nunca solta
na raiz, nunca dois padrões no mesmo arquivo, nunca `docs/` plano:

| Árvore | Padrão | Um arquivo por |
|---|---|---|
| `docs/<idioma>/architecture/` | C4 | nível (contexto, contêiner, componente) |
| `docs/<idioma>/specs/` | SDD | capacidade |
| `docs/<idioma>/decisions/` | ADR (MADR) | decisão, `NNNN-titulo-em-kebab.md` |
| `docs/<idioma>/manual/` | manual | tarefa da pessoa usuária |

Nomes de arquivo **idênticos** nas duas subárvores — tradução é irmão, nunca
bifurcação. Índice em `docs/README.md`. Diagramas são texto (Mermaid).

**ADR é append-only.** ADR aceito é substituído por um novo
(`Status: substituído por NNNN`), nunca reescrito; número nunca se reaproveita.
Template em `templates/adr.md`, spec em `templates/spec.md`.

**Cruze as referências nos dois sentidos.** Todo spec nomeia os ADRs que o
restringem; todo ADR nomeia o nível C4 e os specs que ele move.

**Paridade CLI ↔ GUI.** Tudo que a CLI faz, a GUI também faz, e vice-versa.
Recurso novo entra nas duas ou entra com o motivo escrito de por que só numa.

**Três sistemas operacionais, sempre.** Linux, macOS e Windows são requisito, não
aspiração. Caminho de configuração, socket de IPC, atalho global e captura de
áudio divergem nos três — código que só funciona no Linux é código incompleto.
Ver `docs/pt-BR/specs/portability.md`.

## Superfícies de IA

`.claude/agents/`, `skills/` e `.claude/rules/` são **autorados**. Tudo em
`.claude/skills/`, `.claude/commands/`, `.agents/skills/`, `.codex/` e
`.github/{prompts,instructions}/` é **gerado** por `scripts/sync-ai-surfaces.py`
e carrega um banner na primeira linha. Edite a fonte e rode o gerador:

```bash
python3 scripts/sync-ai-surfaces.py          # projeta
python3 scripts/sync-ai-surfaces.py --check  # falha se houver divergência
```

Traduções são irmãs com sufixo de língua — `AGENTS.en-US.md`, `SKILL.en-US.md`,
`adr.en-US.md` — e o gerador as **ignora**: projetada, uma skill traduzida viraria
uma segunda skill com o mesmo `name`. Elas também não levam frontmatter, para que
nenhum CLI as carregue como definição.
