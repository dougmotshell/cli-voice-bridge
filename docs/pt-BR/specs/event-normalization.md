# Spec — Normalização de eventos

**Capacidade:** transformar os eventos de três CLIs de IA, com três dialetos
diferentes, num único fluxo de "momentos" que a camada de voz entende.

**ADRs que restringem este spec:** [ADR-0004](../decisions/0004-hooks-oficiais-como-transporte-primario.md),
[ADR-0005](../decisions/0005-wrapper-pty-como-transporte-complementar.md),
[ADR-0007](../decisions/0007-esquema-canonico-de-momentos.md).
**Nível C4:** [componente](../architecture/03-component.md) — módulo `core::normalize`.

## Problema

Os três CLIs avisam a pessoa de coisas parecidas — "preciso de permissão",
"terminei", "abri um subagente" — mas com nomes de evento, grafias e campos
diferentes. Se cada adaptador falasse direto com o sintetizador, a política de
voz teria de ser escrita três vezes e ficaria fora de sincronia na primeira
atualização de qualquer um deles.

## Escopo

Dentro: mapear eventos → momentos canônicos; extrair o texto relevante; carimbar
origem, sessão e projeto.
Fora: decidir *se* fala, *como* fala e *com que voz* — isso é
[speech-output](speech-output.md). Fora também: o transporte em si —
[capture-transports](capture-transports.md).

## Momentos canônicos

O vocabulário que o resto do sistema usa. Nada além disto atravessa a fronteira:

| Momento | Significado | Urgência padrão |
|---|---|---|
| `session.started` | Uma sessão começou ou foi retomada | baixa |
| `session.ended` | Sessão terminou | baixa |
| `turn.finished` | O agente terminou de responder e devolveu a vez | **alta** |
| `turn.failed` | O turno acabou por erro | **alta** |
| `decision.needed` | O agente precisa de uma decisão: permissão, escolha, confirmação | **crítica** |
| `input.needed` | O agente está parado esperando texto da pessoa | **crítica** |
| `subagent.started` | Um subagente foi criado | média |
| `subagent.finished` | Um subagente terminou e trouxe resultado | média |
| `task.created` | Uma tarefa entrou na lista | baixa |
| `task.completed` | Uma tarefa foi concluída | média |
| `tool.started` / `tool.finished` | Uma ferramenta vai rodar / rodou | silenciosa por padrão |
| `tool.failed` | Uma ferramenta falhou | média |
| `context.compacting` | O histórico está sendo compactado | baixa |
| `message.text` | Trecho de texto do assistente (narração contínua) | silenciosa por padrão |
| `error` | Erro de sessão, não de ferramenta | **alta** |

Campos de todo momento:

```
momento        um dos nomes acima
origem         "claude" | "codex" | "copilot"
transporte     "hook" | "notify" | "pty" | "stream-json" | "acp"
sessao_id      identificador da sessão no CLI de origem
projeto        diretório de trabalho, para agrupar e para escolher perfil
texto          o que interessa falar, já extraído (pode ser vazio)
detalhe        mapa cru específico da origem, para depuração
recebido_em    carimbo monotônico local
```

`origem` e `transporte` são separados de propósito: o mesmo momento pode chegar
por dois caminhos (hook e PTY) e precisa ser deduplicado — ver *Deduplicação*.

## Mapa: Claude Code → momentos

Fonte: [Hooks reference](https://code.claude.com/docs/en/hooks). Configuração em
`~/.claude/settings.json`, chave `hooks`. Payload em `snake_case` no stdin;
nome do evento em `PascalCase` no campo `hook_event_name`.

| Evento do Claude | Momento | Texto vem de |
|---|---|---|
| `PermissionRequest` | `decision.needed` | `tool_name` + resumo de `tool_input` |
| `Notification` (`matcher: permission_prompt`) | `decision.needed` | `notification_type` |
| `Notification` (`matcher: idle_prompt`) | `input.needed` | — |
| `Elicitation` | `decision.needed` | `mcp_server_name`, `elicitation_type` |
| `Stop` | `turn.finished` | `last_assistant_message` |
| `StopFailure` | `turn.failed` | — |
| `SubagentStart` | `subagent.started` | `agent_type` |
| `SubagentStop` | `subagent.finished` | `last_assistant_message`, `agent_type` |
| `TaskCreated` | `task.created` | `task_name` |
| `TaskCompleted` | `task.completed` | `task_name` |
| `PostToolUseFailure` | `tool.failed` | `tool_name` |
| `PreToolUse` / `PostToolUse` | `tool.started` / `tool.finished` | `tool_name` |
| `MessageDisplay` | `message.text` | `message_text` |
| `PreCompact` | `context.compacting` | — |
| `SessionStart` / `SessionEnd` | `session.started` / `session.ended` | `session_start_reason` / `session_end_reason` |
| `PermissionDenied` | `error` | `tool_name` |

Campos comuns em todo payload: `session_id`, `transcript_path`, `cwd`,
`permission_mode`, `hook_event_name`.

**Atenção.** `Notification` aceita `matcher` por `notification_type`
(`permission_prompt`, `idle_prompt`, `auth_success`, `elicitation_*`, `agent_*`)
— use o matcher em vez de assinar tudo e filtrar depois.

## Mapa: Codex CLI → momentos

Fonte: [Hooks system](https://deepwiki.com/openai/codex/3.11-hooks-system) e
[Advanced configuration](https://developers.openai.com/codex/config-advanced).
Configuração em `~/.codex/hooks.json` ou na tabela `[hooks]` do
`~/.codex/config.toml`; também aceita `.codex/` no repositório.

| Evento do Codex | Momento |
|---|---|
| `PermissionRequest` | `decision.needed` |
| `Stop` | `turn.finished` |
| `SubagentStart` / `SubagentStop` | `subagent.started` / `subagent.finished` |
| `PreToolUse` / `PostToolUse` | `tool.started` / `tool.finished` |
| `UserPromptSubmit` | — (usado só para cortar a fala em curso) |
| `PreCompact` / `PostCompact` | `context.compacting` |
| `SessionStart` / `SessionEnd` | `session.started` / `session.ended` |

Além dos hooks, o Codex tem a chave `notify` no `config.toml`: um argv que ele
executa ao fim de cada turno, acrescentando **um argumento extra** com o JSON do
evento. Campos: `type` (`"agent-turn-complete"`), `last-assistant-message`,
`input-messages`, `thread-id`, `turn-id`, `cwd`. Note o **kebab-case** — é o
único lugar do sistema com essa grafia, e `last-assistant-message` pode faltar e
pode ser muito longo.

TODO: confirmar na versão instalada (0.147.0) quais eventos realmente disparam.
Há relato de que `PreToolUse`/`PostToolUse` só valem para a ferramenta Bash, sem
hooks para escrita de arquivo nem para ferramentas MCP. Verificar com um hook de
log antes de prometer cobertura — usar a skill `map-cli-events`.

## Mapa: Copilot CLI → momentos

Fonte: [Hooks reference](https://docs.github.com/en/copilot/reference/hooks-reference).
Configuração em `.github/hooks/*.json` (repositório) e `~/.copilot/hooks/`
(usuário). Estrutura: `{ "version": 1, "hooks": { "<evento>": [ … ] } }`.
Payload em `camelCase`. Os nomes de evento aceitam duas grafias.

| Evento do Copilot | Momento |
|---|---|
| `permissionRequest` | `decision.needed` |
| `notification` | `decision.needed` ou `input.needed`, conforme `notification_type` |
| `agentStop` | `turn.finished` |
| `subagentStart` / `subagentStop` | `subagent.started` / `subagent.finished` |
| `preToolUse` / `postToolUse` | `tool.started` / `tool.finished` |
| `postToolUseFailure` | `tool.failed` |
| `errorOccurred` | `error` |
| `preCompact` | `context.compacting` |
| `sessionStart` / `sessionEnd` | `session.started` / `session.ended` |

O Copilot não tem evento de "texto exibido". Narração contínua nele só sai por
PTY ou por `--output-format json` — ver [capture-transports](capture-transports.md).

Há ainda a ferramenta `ask_user` (desligável com `--no-ask-user`): quando o
agente pergunta algo, isso chega como `preToolUse` com `toolName == "ask_user"`.
Trate como `input.needed`, não como `tool.started`.

## Deduplicação

O mesmo acontecimento pode chegar duas vezes quando hook e PTY estão ligados no
mesmo CLI. A chave de deduplicação é `(origem, sessao_id, momento, janela)`, com
janela de tolerância configurável (padrão TODO: medir; começar em 1500 ms). O
transporte de maior confiança vence: `hook` > `acp` > `stream-json` > `notify` >
`pty`. O perdedor pode ainda enriquecer o texto do vencedor quando o vencedor
veio vazio.

## Contratos de saída

O normalizador não fala; ele publica no barramento interno do `hookd`. Consumidores:
a fila de voz ([speech-output](speech-output.md)), a GUI e o log de sessão.

## Alternativas consideradas

**Um adaptador por CLI falando direto com o TTS.** Descartado: triplica a
política de voz e a duplicação some do radar até divergir.

**Usar o payload cru sem esquema canônico.** Descartado: obriga a GUI e a
política a conhecerem os três dialetos.

**Só `notify` do Codex e nada de hooks.** Descartado: `notify` só cobre fim de
turno, que é justamente o momento menos urgente da lista.

## Plano de teste

TODO: escrever. O mínimo aceitável é uma tabela-ouro — um payload real capturado
de cada evento de cada CLI, versionado em `crates/core/tests/fixtures/`, e um
teste que afirma o momento resultante. Payload capturado passa por redação antes
de virar fixture: sem caminho de casa, sem nome de cliente, sem token.

## Questões em aberto

- Cobertura real dos hooks de ferramenta do Codex 0.147.0 (acima).
- O Claude tem 30 eventos; nem todos foram mapeados aqui. Falta decidir o que
  fazer com `TeammateIdle`, `FileChanged`, `WorktreeCreate/Remove`,
  `ConfigChange`, `InstructionsLoaded`, `ElicitationResult`, `PostToolBatch`,
  `UserPromptExpansion`, `Setup`, `CwdChanged`, `DirectoryAdded`.
- Como versionar o mapa quando um CLI renomear um evento sem aviso. Provável
  resposta: momento desconhecido vira `error` com o nome cru, e o `cvb doctor`
  reclama.
