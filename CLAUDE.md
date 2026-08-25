# CLAUDE.md — adaptador do Claude Code

@AGENTS.md

`AGENTS.md` é o contrato canônico deste projeto e vale integralmente aqui. O que
segue é só o que é específico do Claude Code.

## Subagentes disponíveis

| Agente | Para quê |
|---|---|
| `cli-event-cartographer` | Levantar e conferir os eventos de interação de um CLI de IA e mantê-los em dia no `docs/pt-BR/specs/event-normalization.md` |
| `voice-pipeline-doctor` | Diagnosticar a cadeia de áudio fim a fim: dispositivo, captura, STT, sidecar TTS, reprodução |

## Comandos de barra gerados

`/new-adr`, `/new-spec`, `/map-cli-events`, `/smoke-voice`. As fontes são
`skills/<nome>/SKILL.md`; os arquivos em `.claude/commands/` e `.claude/skills/`
são gerados.

## Servidores MCP

Nenhum específico deste projeto. TODO: decidir se o `hookd` expõe uma superfície
MCP para os próprios agentes consultarem a fila de fala.

## Regra do gerador

Nunca edite arquivo com o banner `managed-by:cli-voice-bridge/sync-ai-surfaces`.
Edite a fonte em `.claude/agents/`, `skills/` ou `.claude/rules/` e rode
`python3 scripts/sync-ai-surfaces.py`.

## Ao mexer nos hooks deste projeto

Este repositório instala hooks do Claude Code em `~/.claude/settings.json`. A
máquina de desenvolvimento já tem hooks de terceiros lá (`rtk`); a de outra
pessoa pode não ter nada, nem o arquivo. Ler antes de escrever, compor em vez de
substituir, criar do zero quando não houver — e testar os dois cenários. Ver a
armadilha correspondente em `AGENTS.md`.
