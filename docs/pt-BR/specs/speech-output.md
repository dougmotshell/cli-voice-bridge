# Spec — Saída de voz

**Capacidade:** transformar momentos em fala, na voz clonada, sem virar ruído.

**ADRs que restringem este spec:** [ADR-0003](../decisions/0003-tts-delegado-ao-voice-clone.md),
[ADR-0007](../decisions/0007-esquema-canonico-de-momentos.md),
[ADR-0009](../decisions/0009-reproducao-por-reprodutor-do-sistema.md).
**Nível C4:** [componente](../architecture/03-component.md) — `hookd::speech`.

## Estado

| Parte | Estado |
|---|---|
| Redação | **implementada** — `speech::redact`, com testes de isca |
| Moldes e modo discreto | **implementados** — `speech::template` |
| Ponte com o sidecar | **implementada** — `core::sidecar` |
| Cache de frases | **implementado** — indexado por (voz, idioma, texto) |
| Reprodução e degradação | **implementadas** — `core::audio` ([ADR-0009](../decisions/0009-reproducao-por-reprodutor-do-sistema.md)) |
| Política por urgência | **parcial** — `sempre` e `nunca` valem; `ausente` cala, por falta de detecção de presença |
| Fila com prioridade, colapso, expiração e corte | **implementada** — `speech::queue` |
| Resumo de mensagem longa | **não existe** — hoje só um corte por número de caracteres |
| Detecção de presença | **não existe** — é o que falta para `ausente` valer |

## Problema

Um turno do agente gera dezenas de momentos. Falar todos é insuportável; falar
poucos demais derrota o propósito. E o XTTS-v2 em CPU não é instantâneo — a
síntese leva um tempo comparável à duração do áudio, então a fila importa.

## Escopo

Dentro: política de o que falar, fila e prioridade, corte de fala em curso
(*barge-in*), redação de conteúdo sensível, ponte com o `voice-clone`, cache.
Fora: como o momento chegou ([capture-transports](capture-transports.md)) e a
resposta da pessoa ([speech-input](speech-input.md)).

## Política: o que vira fala

Padrão por urgência, tudo sobrescrevível por CLI, por projeto e por momento
([configuration](configuration.md)):

| Urgência | Momentos | Comportamento padrão |
|---|---|---|
| crítica | `decision.needed`, `input.needed` | fala sempre, interrompe o que estiver falando |
| alta | `turn.finished`, `turn.failed`, `error` | fala, entra na frente da fila |
| média | `subagent.*`, `task.completed`, `tool.failed` | fala só se a pessoa estiver ausente ou a sessão em segundo plano |
| baixa | `session.*`, `task.created`, `context.compacting` | não fala; aparece na GUI |
| silenciosa | `tool.started`, `tool.finished`, `message.text` | não fala, salvo modo narração |

**Presença.** "Ausente" é o critério que evita o pior do ruído: se a janela do
terminal está em foco e houve tecla há poucos segundos, a pessoa já viu na tela e
não precisa ouvir. TODO: definir a detecção de foco nos três SOs, e o que fazer
quando não dá para saber (provável: assumir presente, falar menos).

**Modo narração** (opt-in): fala `message.text` enquanto o assistente escreve.
Só faz sentido com transporte `pty` ou `stream-json`.

## Texto: do momento à frase

Nenhum momento é falado cru. A cadeia é:

1. **Redação.** Remove segredo antes de qualquer outra coisa — token, chave,
   senha, e o que a configuração marcar como sensível. Um caminho absoluto vira
   o nome do arquivo. Ver *Privacidade*.
2. **Molde.** Cada momento tem um molde curto em pt-BR, editável pela pessoa:
   `decision.needed` → "O Claude quer rodar {ferramenta}. Autorizo?";
   `turn.finished` → "{cli} terminou. {resumo}".
3. **Resumo.** `last_assistant_message` costuma ter parágrafos. Falar tudo é
   inútil. Resumir para uma ou duas frases. TODO: decidir o resumidor —
   extrativo local (rápido, sem dependência) ou um modelo pequeno local. Chamar
   um modelo de nuvem para isso contraria o princípio de não vazar contexto.
4. **Limite duro.** Nenhuma fala passa de N segundos (padrão TODO: começar em 12).
   O resto fica na GUI e sai com "tem mais na tela".

## Fila

Uma fila só, com prioridade e um único reprodutor.

- Crítica **corta** o que está tocando e limpa o que for de urgência menor.
- Alta entra na frente das de média e baixa.
- Momentos da mesma sessão e mesmo momento **colapsam**: três `tool.failed`
  seguidos viram "três ferramentas falharam".
- Momento que envelheceu mais que a janela de relevância é descartado sem falar
  — anunciar "terminei" 40 segundos depois é pior que silêncio.
- `UserPromptSubmit` (qualquer CLI) chega como `user.returned` e **corta tudo**:
  a fala em curso e o que estiver na fila. Se a pessoa está digitando, já voltou.
- `cvb mute` também corta na hora, e não só a partir da próxima fala.

**Cortar é matar o processo do reprodutor**, que é o que a escolha do
[ADR-0009](../decisions/0009-reproducao-por-reprodutor-do-sistema.md) permite. O
`Voz` guarda o processo em curso justamente para isso; quem espera pelo fim faz
isso por sondagem, porque bloquear no processo o seguraria contra quem tenta
matá-lo.

## Ponte com o voice-clone

O `voice-clone` é dependência externa somente leitura, chamada pelo contrato de
CLI dele, nunca por import. O caminho vem da configuração.

```
<venv do voice-clone>/bin/python falar.py falar <voz> "<texto>" --saida <arquivo> [--rapido]
```

O sidecar mantém o processo Python **vivo** entre as chamadas: carregar o XTTS-v2
leva cerca de 30 segundos, e pagar isso por frase é inaceitável. Portanto o
sidecar não é um `spawn` por fala — é um servidor local de vida longa com uma
fila própria, e o `hookd` fala com ele por socket.

TODO: `falar.py` hoje é um comando de um tiro. Ou o sidecar embrulha `vozclone.py`
num laço servidor (mexe no `voice-clone`, o que exige conversa separada), ou o
sidecar é escrito aqui e importa `vozclone` do venv de lá. A segunda opção
respeita o "somente leitura" e é a hipótese de trabalho.

**Degradação.** Sidecar morto ou XTTS indisponível não pode calar o sistema: cai
para a voz do sistema operacional (`espeak-ng`/`say`/SAPI) com aviso, e o
`cvb doctor` diz o porquê. O `cvb say` mostra por qual caminho falou — "voz
clonada" e "voz do sistema" são resultados bem diferentes, e confundi-los
esconderia um sidecar morto. Falar com voz feia é melhor que não falar que o agente
está travado esperando permissão.

**Cache.** Frases fixas ("terminei", "preciso de permissão") são poucas e se
repetem. Sintetizar uma vez, guardar o WAV indexado por (voz, idioma, texto).
Isso troca a maior parte das falas por reprodução instantânea. Fica em
`<dados>/cache-audio/`, com nome derivado de um hash não criptográfico — é um
nome de arquivo, não uma garantia de integridade.

TODO: o cache não tem limite nem expiração. Precisa de um teto antes que o uso
prolongado o transforme num problema.

## Privacidade

Falar é publicar num ambiente compartilhado. Quem está na sala ouve o nome do
cliente, o caminho do repositório, o trecho de código.

- Redação roda **antes** do molde, sempre, sem opção de desligar para segredos.
- Modo discreto (`cvb quiet`) fala só a categoria, nunca o conteúdo: "o agente
  precisa de uma decisão" em vez de dizer qual comando.
- Nenhum texto de momento é gravado em disco além do log de sessão, que é
  ignorado pelo git e tem retenção configurável.
- Nenhum áudio, texto ou transcrição sai da máquina.

## Alternativas consideradas

**Falar direto do processo do hook.** Descartado: bloqueia o agente pelo tempo da
síntese e não tem como cortar a fala anterior.

**Um TTS de nuvem (ElevenLabs, OpenAI).** Descartado: manda o texto do trabalho
para fora, custa dinheiro, e a voz clonada — o motivo do projeto — já existe
localmente.

**Só a voz do sistema, sem XTTS.** Descartado como padrão, mantido como fallback.

## Plano de teste

TODO: escrever. Mínimo: teste de fila (crítica corta média), teste de colapso,
teste de redação com uma bateria de segredos-isca, e um teste de fumaça manual
(`cvb say`) porque só ouvido julga naturalidade.

## Questões em aberto

- Qual resumidor (acima).
- Detecção de presença/foco nos três SOs (acima).
- O que fazer com vários CLIs falando ao mesmo tempo: vozes diferentes por
  ferramenta, ou prefixo falado ("Codex:")? Vozes diferentes é mais agradável e
  mais caro de configurar.
