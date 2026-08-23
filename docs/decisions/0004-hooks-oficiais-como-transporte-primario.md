# ADR-0004 — Hooks oficiais como transporte primário

**Status:** aceito — 2026-08-23
**Nível C4:** [contêiner](../architecture/02-container.md)
**Specs que esta decisão move:** [capture-transports](../specs/capture-transports.md), [event-normalization](../specs/event-normalization.md)

## Contexto

A exigência é funcionar em qualquer terminal ou shell — inclusive o terminal
integrado do VS Code e do IntelliJ, `tmux` e sessões remotas. Os três CLIs
oferecem sistemas de hook:

- Claude Code 2.1.241 — cerca de 30 eventos, em `~/.claude/settings.json`
- Codex CLI 0.147.0 — `~/.codex/hooks.json` mais a chave `notify`
- Copilot CLI 1.0.80 — `.github/hooks/*.json` e `~/.copilot/hooks/`

Esta máquina já usa hooks de terceiros (`rtk` em `PreToolUse`, no Claude e no
Codex).

## Decisão

Hooks são o transporte primário. Todo momento que um hook cobre chega por hook.
Os demais transportes existem para o que o hook não alcança.

A instalação **compõe** com o que já existe: lê o arquivo, acrescenta a entrada e
preserva o resto. `--dry-run` mostra o diff antes de escrever, e desinstalar
remove só o que foi instalado.

Hook que falha sai com código 0 e em silêncio. A única exceção é a decisão de
permissão por voz, e mesmo ela cai para a tela quando não há confiança.

## Consequências

**Boas.** Indiferente ao terminal por construção: quem executa o hook é o
processo do CLI. Payload estruturado em JSON, sem parsing de tela. Não quebra a
cada mudança de TUI.

**Ruins.** Cobertura limitada ao que cada fornecedor decidiu expor — e os três
expõem conjuntos diferentes, com três dialetos de nomenclatura. O Codex guarda um
`trusted_hash` do hook no `config.toml`, então mudar o comando exige
reconfirmação da pessoa, e um instalador silencioso deixaria o hook inerte.

**Restringe.** O `hookc` tem de ser rápido ([ADR-0001](0001-nucleo-em-rust-com-cliente-de-hook-separado.md))
e o instalador nunca pode reescrever arquivo de configuração alheio por inteiro.

## Alternativas

**Ler o transcript da sessão.** Descartado: assíncrono, formato não é contrato
público, e não diz *quando* a pessoa é necessária.

**Só wrapper PTY.** Descartado como primário: frágil e obriga a abrir os CLIs de
um jeito diferente. Virou [ADR-0005](0005-wrapper-pty-como-transporte-complementar.md).

**Só ACP/app-server.** Descartado como primário: custa a TUI original. Fica como
modo opcional.
