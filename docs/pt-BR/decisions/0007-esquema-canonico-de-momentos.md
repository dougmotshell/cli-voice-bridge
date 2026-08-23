# ADR-0007 — Um esquema canônico de "momentos" entre os CLIs e a voz

**Status:** aceito — 2026-08-23
**Nível C4:** [componente](../architecture/03-component.md)
**Specs que esta decisão move:** [event-normalization](../specs/event-normalization.md), [speech-output](../specs/speech-output.md), [configuration](../specs/configuration.md)

## Contexto

Os três CLIs avisam de coisas parecidas com nomes e formatos diferentes: Claude
usa `snake_case` no payload e `PascalCase` no evento; Copilot usa `camelCase` e
aceita duas grafias de evento; o `notify` do Codex usa `kebab-case`. Os conjuntos
de eventos não coincidem, e cada fornecedor muda o seu quando quer.

## Decisão

Existe um vocabulário fechado de **momentos** — `decision.needed`,
`turn.finished`, `subagent.started`, e assim por diante — e é a única coisa que
atravessa a fronteira entre os adaptadores e o resto do sistema. Cada adaptador
traduz o dialeto do seu CLI; política, fila, GUI e configuração só conhecem
momentos.

Evento desconhecido não é descartado: vira `error` carregando o nome cru, e o
`cvb doctor` reclama. É assim que uma renomeação silenciosa por um fornecedor
aparece em vez de sumir.

## Consequências

**Boas.** A política de voz é escrita uma vez. A configuração fala em momentos,
que é o que a pessoa entende — "quando precisar de decisão, sempre fale" vale nos
três CLIs. Adicionar um quarto CLI é escrever um adaptador, não mexer no núcleo.

**Ruins.** Uma camada de tradução para manter, e detalhe específico de um CLI que
não cabe no vocabulário fica no campo `detalhe`, subaproveitado. Momento que
existe só num CLI (`MessageDisplay`, do Claude) gera comportamento assimétrico
que precisa ser documentado, não escondido.

**Restringe.** Nenhum consumidor pode ler o payload cru para tomar decisão. Se
precisou, ou falta um momento no vocabulário, ou falta um campo no esquema — as
duas coisas se resolvem estendendo o esquema, não furando a fronteira.

## Alternativas

**Cada adaptador falando direto com o TTS.** Descartado: triplica a política e a
divergência some do radar até quebrar.

**Passar o payload cru adiante.** Descartado: obriga a GUI e a configuração a
conhecerem os três dialetos, e amarra a interface ao formato de terceiros.

**Adotar o vocabulário de um dos três como canônico** (o do Claude, que é o mais
rico). Descartado: prende o projeto às escolhas de nomenclatura de um fornecedor
e fica esquisito quando ele renomear.
