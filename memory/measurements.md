# Medições

Números medidos nesta máquina. Servem para não redecidir por intuição o que já
foi medido — e para saber quando uma mudança piorou algo.

## Latência do cliente de hook — 2026-08-23

O ponto do ADR-0001 é que o `hookc` roda em série com o agente de IA, centenas
de vezes por sessão, e por isso precisa ser quase gratuito. Agora está medido:

| Medida | Valor |
|---|---|
| 200 invocações do `cvb-hook` (release), com daemon de pé | 387 ms |
| **Média por chamada** | **1,94 ms** — e isso inclui o `fork` do shell e o pipe |
| `cvb-hook` sem daemon (caminho de falha) | sai em 0, silencioso |

Referência de comparação: o arranque do Python com os imports do `voice-clone`
fica em dezenas de milissegundos, antes de qualquer trabalho útil. A ordem de
grandeza é a que o ADR-0001 previu.

**Se algum dia isso subir**, o suspeito é dependência nova no `hookc`. O crate
não tem `clap` de propósito: para três argumentos posicionais, o custo de
arranque dele não se paga.

## Tamanho dos binários (release, Linux x86_64) — 2026-08-23

Perfil `opt-level = "z"`, `lto`, `strip`, `panic = "abort"`.

| Binário | Tamanho |
|---|---|
| `cvb-hook` | 340 KB |
| `cvb-hookd` | 424 KB |
| `cvb` | 612 KB (é o único com `clap`) |
| `cvb-ptywrap` | 288 KB (ainda é só a mensagem de não implementado) |

## Ainda não medido

- Latência e taxa de erro do STT em pt-BR — é o que fecha o
  [ADR-0006](../docs/decisions/0006-stt-offline-na-maquina.md), e a decisão do
  motor sai daí, não de tabela comparativa.
- Tempo entre o evento chegar e o primeiro som sair. É o número que a pessoa
  sente; hoje não existe porque a síntese não está ligada.
