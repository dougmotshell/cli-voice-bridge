# Spec — Transportes de captura

**Capacidade:** receber o que os CLIs de IA têm a dizer, em qualquer terminal,
shell ou IDE, sem depender de nenhum deles em particular.

**ADRs que restringem este spec:** [ADR-0004](../decisions/0004-hooks-oficiais-como-transporte-primario.md),
[ADR-0005](../decisions/0005-wrapper-pty-como-transporte-complementar.md),
[ADR-0008](../decisions/0008-ipc-por-socket-local.md).
**Nível C4:** [contêiner](../architecture/02-container.md).

## Problema

A pessoa usa esses CLIs em lugares diferentes: terminal do sistema, terminal
integrado do VS Code, do IntelliJ, dentro de `tmux`, por `ssh`, no PowerShell.
Um mecanismo que dependa de ler a tela, de um emulador específico ou de estar em
primeiro plano falha na maioria desses lugares.

## A escolha que resolve isso

**Hooks rodam no processo do CLI, não no terminal.** Quando o Claude Code dispara
`PermissionRequest`, quem executa o comando do hook é o próprio processo do
Claude — dá no mesmo se ele foi aberto no Windows Terminal, no painel do IntelliJ
ou numa sessão `ssh` sem TTY. É por isso que o hook é o transporte primário: é o
único que é indiferente ao terminal por construção.

O corolário é que a saída do hook **não pode** ser a fala. O hook só entrega o
evento a um daemon que já está rodando; quem fala é o daemon, com o dispositivo
de áudio da sessão de desktop. Ver [ADR-0001](../decisions/0001-nucleo-em-rust-com-cliente-de-hook-separado.md).

## Os cinco transportes

| Transporte | Cobertura | Confiança | Depende do terminal? |
|---|---|---|---|
| `hook` | eventos estruturados dos 3 CLIs | alta | não |
| `notify` | só fim de turno, só Codex | alta | não |
| `stream-json` | tudo, só em modo não interativo (`-p`) | alta | não |
| `acp` | tudo, mas substitui a TUI | alta | não se aplica |
| `pty` | tudo, inclusive o que só existe na tela | baixa | sim, é o terminal |

### hook — primário

Instalado por `cvb install`. Um comando por evento apontando para o `hookc`.

- **Claude Code:** `~/.claude/settings.json` → `hooks.<Evento>[].hooks[]`, tipo
  `command`. Aceita `matcher`, `timeout`, `async`. Suporta `${CLAUDE_PROJECT_DIR}`.
- **Codex CLI:** `~/.codex/hooks.json` (ou `[hooks]` no `config.toml`), mais a
  chave `notify` para o fim de turno. O Codex guarda um `trusted_hash` do hook em
  `config.toml` — mudar o comando exige reconfirmação; o instalador precisa
  avisar disso em vez de deixar o hook silenciosamente inerte.
- **Copilot CLI:** `~/.copilot/hooks/*.json` (usuário) e `.github/hooks/*.json`
  (repositório), com `{"version": 1, "hooks": {…}}`.

**Composição, não substituição.** Esta máquina já tem `rtk hook claude` em
`PreToolUse` no Claude e no Codex. `cvb install` lê o arquivo, acrescenta a
entrada do `cvb` e preserva o resto; `cvb install --dry-run` mostra o que mudaria
e `--diff` mostra linha a linha. Desinstalar remove só o que instalou, e o
original de cada arquivo fica em `*.cvb-backup`.

**Como uma entrada é reconhecida como nossa:** o comando dela menciona o binário
`cvb-hook`. JSON não tem comentário, então não há como carimbar de outro jeito —
e é isto que `cvb uninstall` usa. Reinstalar remove a entrada anterior antes de
pôr a nova, para nunca duplicar.

**Ordem das chaves preservada.** O `serde_json` roda com `preserve_order`; sem
isso, regravar o `settings.json` embaralharia a configuração de quem já estava
lá. O conteúdo é preservado, mas a formatação é normalizada — daí o backup.

**O Copilot não precisa de composição.** `~/.copilot/hooks/` é um diretório de
arquivos JSON, então o `cvb` escreve o seu próprio (`cli-voice-bridge.json`) e
nunca encosta em arquivo de terceiro. Instalar é escrever; desinstalar é apagar.

### Quais eventos são assinados

| CLI | Eventos |
|---|---|
| Claude | `PermissionRequest`, `Notification` (matcher `permission_prompt\|idle_prompt`), `Elicitation`, `Stop`, `StopFailure`, `SubagentStart`, `SubagentStop`, `TaskCompleted`, `PostToolUseFailure`, `SessionStart`, `SessionEnd` |
| Codex | `PermissionRequest`, `Stop`, `SubagentStart`, `SubagentStop`, `UserPromptSubmit`, `SessionStart`, `SessionEnd` |
| Copilot | `permissionRequest`, `notification`, `agentStop`, `subagentStart`, `subagentStop`, `postToolUseFailure`, `errorOccurred`, `preToolUse` (matcher `ask_user`), `sessionStart`, `sessionEnd` |

**`PreToolUse` e `PostToolUse` ficam de fora no Claude e no Codex, de propósito.**
São momentos silenciosos por padrão, e `PreToolUse` é o caminho quente onde o
`rtk` já mora — assinar os dois custaria em toda chamada de ferramenta para não
falar nada. Quem quiser narração liga na configuração e acrescenta o hook à mão.

No Copilot, `preToolUse` entra **com matcher `ask_user`**, porque é assim que
aquele agente faz pergunta: sem isso, o momento em que a pessoa é necessária
passaria despercebido.

### notify — complementar, só Codex

`notify = ["<caminho do hookc>", "--origem", "codex", "--transporte", "notify"]`
no `config.toml`. O Codex acrescenta o JSON como argumento final. O `hookc` já
aceita essa forma (payload no argumento em vez do stdin).

**O `cvb install` não mexe nisso.** Editar o `config.toml` do Codex exigiria um
editor de TOML que preserve formatação e comentários — o arquivo tem dezenas de
entradas de projeto, servidores MCP e estado de hook, e regravá-lo com um
serializador comum apagaria a formatação de quem escreveu. Como o `notify` é
redundante com o hook `Stop`, o custo não se paga por ora.

TODO: se entrar, é com `toml_edit`, e com o mesmo `--dry-run` do resto.

### stream-json — modo não interativo

`claude -p --output-format stream-json`, `codex exec --json`,
`copilot -p --output-format json`. Útil para agentes de fundo e para tarefas
agendadas, onde não há TUI nenhuma. Não substitui os hooks no uso interativo.

### acp — protocolo de agente

O Copilot CLI expõe `--acp` (Agent Client Protocol); o Codex tem `app-server` e
`mcp-server`; o Claude Code tem o Agent SDK e `--input-format stream-json`. É o
caminho mais limpo para voz de ida e volta, porque a resposta falada entra como
mensagem de protocolo em vez de teclas simuladas. O custo é abandonar a TUI
original. Fica como modo opcional: `cvb console --cli copilot`.

TODO: verificar a paridade de recursos. Um agente por ACP pode não expor tudo que
a TUI expõe (modos de permissão, `/comandos`, plugins).

### pty — o que só existe na tela

`cvb wrap -- claude` abre o CLI dentro de um pseudo-terminal e repassa tudo, nos
dois sentidos, transparente. Isso captura o que nenhum hook entrega: o texto que
o assistente está escrevendo agora, o desenho do menu de permissão, a pergunta
com opções. E é o que permite **injetar** a resposta falada direto no `stdin` do
CLI, sem simular teclado no sistema.

Funciona em qualquer terminal porque o wrapper *é* o terminal do ponto de vista
do CLI. Funciona no terminal integrado do VS Code e do IntelliJ do mesmo jeito,
desde que a pessoa abra o CLI pelo wrapper.

É frágil por natureza: qualquer redesenho de TUI muda o que se vê. Por isso é
opcional, ligado por CLI na configuração, e **nunca** é a única fonte de um
momento que o hook já cobre. O que só ele cobre — narração e injeção de texto —
degrada com aviso quando o parsing falha, não em silêncio.

TODO: decidir a estratégia de parsing. Provável: `vte`/`anstyle` para reconstruir
a tela lógica e regras por CLI versionadas junto do número de versão detectado,
com um `cvb doctor --pty` que testa as regras contra a versão instalada.

## Alcance por sistema operacional

| | Linux | macOS | Windows |
|---|---|---|---|
| hook | sim | sim | sim (comando via `cmd`/`pwsh`) |
| pty | `openpty` | `openpty` | ConPTY |
| socket de IPC | socket UNIX | socket UNIX | named pipe |
| atalho global | portal `xdg-desktop-portal` (Wayland) / X11 | precisa de permissão de Acessibilidade | `RegisterHotKey` |

Ver [portability](portability.md).

## Alternativas consideradas

**Ler o arquivo de transcript.** O Claude expõe `transcript_path` e o Codex tem
`~/.codex/sessions/`. Descartado como transporte primário: é assíncrono, o
formato não é contrato público e não diz *quando* a pessoa é necessária — que é
exatamente a informação que este projeto existe para dar.

**Assistir à saída do terminal por acessibilidade do SO.** Descartado: exige
permissões invasivas, quebra em terminal remoto e não funciona por `ssh`.

**Só ACP, sem TUI.** Descartado como padrão: a pessoa perde as TUIs que já usa.
Continua disponível como modo.

## Plano de teste

TODO: escrever. Precisa cobrir, no mínimo: instalação de hook sobre um
`settings.json` que já tem hooks de terceiros (asserção: o hook alheio continua
lá); desinstalação (asserção: volta ao estado anterior byte a byte); e um teste
de fumaça por CLI que dispara um evento real e confere que chegou ao daemon.

## Questões em aberto

- Cobertura real dos hooks de ferramenta do Codex — ver
  [event-normalization](event-normalization.md).
- Como o daemon descobre o dispositivo de áudio certo quando a sessão do CLI está
  numa máquina e a pessoa está noutra (`ssh`). Provável: encaminhar o evento para
  o daemon da máquina local por um transporte explícito, não adivinhar.
