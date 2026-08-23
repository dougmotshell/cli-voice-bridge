<!-- managed-by:cli-voice-bridge/sync-ai-surfaces — do not edit by hand -->
---
applyTo: "crates/**/*.rs"
---
<!-- fonte: .claude/rules/rust-sources.md -->

# Código Rust deste projeto

**O `hookc` é caminho quente.** Ele roda em série com o agente de IA, centenas de
vezes por sessão. Nada de carregar modelo, abrir rede, ler configuração pesada ou
fazer parsing além do necessário para rotear. Dependência nova no `hookc` precisa
de justificativa; no `hookd` não.

**Falha do hook nunca trava o agente.** No caminho do `hookc`, todo erro vira
saída com código 0 e silêncio. `unwrap()` e `expect()` aí são defeito.

**`core` não depende de `adapters`.** A seta é sempre `adapters → core`. É o que
permite acrescentar um quarto CLI sem tocar no núcleo
([ADR-0007](../../docs/decisions/0007-esquema-canonico-de-momentos.md)).

**Payload de terceiro é dado, não contrato.** Campo ausente ou tipo inesperado é
normal — os CLIs mudam sem avisar. Desserialize com tolerância e transforme o
desconhecido em `error` com o nome cru, nunca em pânico.

**Nada de caminho fixo de plataforma.** Configuração, socket, cache e log saem de
funções que conhecem os três sistemas. Um literal `~/.config` no código é
defeito ([portability](../../docs/specs/portability.md)).

**Segredo não chega ao log.** `redact` roda antes do molde e antes de qualquer
escrita em disco. Se você está logando um payload cru para depurar, é temporário
e sai antes do commit.

**Prosa em pt-BR, identificadores em en-US.** Comentários, mensagens de erro e
documentação de módulo em português com acentuação completa; nomes de função,
tipo e variável em inglês.

`cargo clippy -- -D warnings` passa antes do commit.
