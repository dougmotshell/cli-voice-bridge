# Superfícies de eventos dos CLIs de IA

Verificado em **2026-08-23**, nesta máquina. O mapa detalhado está em
`docs/specs/event-normalization.md`; aqui fica só o que é fácil esquecer.

## Versões conferidas

| CLI | Versão |
|---|---|
| Claude Code | 2.1.241 |
| Codex CLI | 0.147.0 |
| Copilot CLI | 1.0.80 |

## O que é fácil esquecer

**Os três usam grafias diferentes.** Claude: evento em `PascalCase`, payload em
`snake_case`. Copilot: `camelCase` nos dois, e aceita duas grafias de evento.
O `notify` do Codex: `kebab-case` (`last-assistant-message`) — é o único lugar do
sistema com isso.

**O Codex guarda um `trusted_hash` do comando de hook** em `config.toml`, na
seção `[hooks.state]`. Mudar o comando deixa o hook inerte até a pessoa
confirmar numa sessão. Isso já vai parecer "o evento não dispara" pelo menos uma
vez.

**Esta máquina já tem hooks de terceiros:** `rtk hook claude` em `PreToolUse`, no
Claude e no Codex. Qualquer instalador tem de compor, nunca substituir.

**O Copilot não tem evento de texto exibido.** Narração contínua nele só sai por
PTY ou por `--output-format json`.

**A ferramenta `ask_user` do Copilot** é como o agente faz perguntas. Chega como
`preToolUse` com `toolName == "ask_user"` — é `input.needed`, não
`tool.started`. Fácil de classificar errado.

## Não verificado ainda

- Se o Codex realmente só dispara hooks de ferramenta para Bash (relato de
  terceiros, não confirmado nesta versão).
- Cerca de metade dos ~30 eventos do Claude Code.

Use a skill `map-cli-events` para fechar essas lacunas — e atualize esta página
com a versão em que verificou.
