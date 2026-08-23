# C4 nível 3 — Componentes do `hookd`

O que existe dentro do daemon. As peças ao redor estão no
[nível 2](02-container.md).

```mermaid
graph TB
    subgraph entrada["Entrada"]
        ipc["ipc::server<br/>aceita hookc, ptywrap,<br/>cvb e GUI"]
        adapters["adapters::{claude, codex, copilot}<br/>um por CLI, conhece o dialeto"]
    end

    normalize["core::normalize<br/>payload → momento canônico<br/>+ deduplicação"]
    policy["policy<br/>fala? com que urgência?<br/>presença, perfil, projeto"]
    redact["redact<br/>remove segredo e encurta caminho"]
    template["template<br/>momento → frase em pt-BR"]
    summarize["summarize<br/>encurta mensagem longa"]
    queue["speech::queue<br/>prioridade, colapso,<br/>corte, expiração"]
    tts["speech::tts<br/>ponte com o sidecar<br/>+ cache + fallback do SO"]
    play["audio::playback"]

    subgraph escuta["Escuta"]
        hotkey["listen::hotkey<br/>atalho global"]
        capture["audio::capture"]
        vad["listen::vad"]
        stt["listen::stt<br/>motor TODO"]
        resolve["listen::resolve<br/>vocabulário fechado<br/>ou ditado"]
    end

    reply["reply<br/>devolve ao CLI: resposta de hook,<br/>stdin do PTY, clipboard ou protocolo"]
    state["state<br/>sessões vivas, log, métricas"]
    config["core::config<br/>camadas e recarga a quente"]

    ipc --> adapters --> normalize --> policy
    policy -->|"vale falar"| redact --> template --> summarize --> queue --> tts --> play
    policy -->|"precisa de resposta"| hotkey
    hotkey --> capture --> vad --> stt --> resolve --> reply
    normalize --> state
    config -.->|"lê"| policy
    config -.-> template
    config -.-> tts
    config -.-> resolve
    queue -.->|"corta a fala em curso"| play
```

## Componentes

| Componente | Responsabilidade | Spec |
|---|---|---|
| `ipc::server` | Aceitar conexões dos clientes locais | [interfaces](../specs/interfaces.md) |
| `adapters::*` | Traduzir o dialeto de cada CLI. Um módulo por CLI | [event-normalization](../specs/event-normalization.md) |
| `core::normalize` | Momento canônico e deduplicação entre transportes | [event-normalization](../specs/event-normalization.md) |
| `policy` | Decidir se fala, com que urgência, considerando presença e perfil | [speech-output](../specs/speech-output.md) |
| `redact` | Tirar segredo antes de qualquer outra coisa | [speech-output](../specs/speech-output.md) |
| `template` / `summarize` | Momento vira frase curta em pt-BR | [speech-output](../specs/speech-output.md) |
| `speech::queue` | Prioridade, colapso de repetidos, corte, expiração | [speech-output](../specs/speech-output.md) |
| `speech::tts` | Sidecar, cache de frases fixas, fallback para a voz do SO | [speech-output](../specs/speech-output.md) |
| `listen::*` | Atalho, captura, VAD, transcrição, resolução | [speech-input](../specs/speech-input.md) |
| `reply` | Entregar a resposta pelo caminho certo de cada CLI | [speech-input](../specs/speech-input.md) |
| `core::config` | Camadas de configuração e recarga a quente | [configuration](../specs/configuration.md) |
| `state` | Sessões vivas, log com retenção, métricas para a GUI | — |

## Dependências entre componentes

`adapters` depende de `core`; `core` não depende de `adapters`. É o que permite
acrescentar um quarto CLI sem tocar no núcleo
([ADR-0007](../decisions/0007-esquema-canonico-de-momentos.md)).

`redact` roda **antes** de `template` e de qualquer log. Segredo não pode chegar
ao molde nem ao disco.

## Nível 4 (código)

Deliberadamente ausente. Não há código ainda, e mesmo depois um diagrama de
classes não paga o custo de manutenção num projeto deste tamanho. Se algum
módulo ficar intrincado o bastante para justificar, cria-se `04-code.md` só para
ele — não para o sistema todo.
