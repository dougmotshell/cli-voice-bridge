# Instruções do Copilot — cli-voice-bridge

@AGENTS.md

`AGENTS.md` é o contrato canônico deste projeto e vale integralmente aqui. O que
segue é só o que é específico do Copilot CLI.

## Prompts gerados

`.github/prompts/*.prompt.md` — `new-adr`, `new-spec`, `map-cli-events`,
`smoke-voice`. As fontes são `skills/<nome>/SKILL.md`.

## Instruções por caminho

`.github/instructions/*.instructions.md` são geradas a partir de
`.claude/rules/*.md` (o `paths:` da fonte vira `applyTo:`).

## Servidores MCP

Nenhum específico deste projeto.

## Regra do gerador

Nunca edite arquivo com o banner `managed-by:cli-voice-bridge/sync-ai-surfaces`.
Edite a fonte e rode `python3 scripts/sync-ai-surfaces.py`.

## Ao mexer nos hooks deste projeto

Os hooks do Copilot CLI ficam em `.github/hooks/*.json` (repositório) e em
`~/.copilot/hooks/` (usuário). Nomes de evento em `camelCase` e payload em
`camelCase` — dialeto diferente do Claude e do Codex. Ver
`docs/pt-BR/specs/event-normalization.md` antes de assumir qualquer campo.
