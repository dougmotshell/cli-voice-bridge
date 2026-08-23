# Documentação (pt-BR)

Esta é a **fonte da verdade**. Escreva aqui primeiro; a tradução em
[`../en-US/`](../en-US/) acompanha, com os mesmos nomes de arquivo.

| Árvore | Padrão | Um arquivo por | Índice |
|---|---|---|---|
| [`architecture/`](architecture/) | C4 | nível | [README](architecture/README.md) |
| [`specs/`](specs/) | SDD | capacidade | abaixo |
| [`decisions/`](decisions/) | ADR (MADR) | decisão | [README](decisions/README.md) |
| [`manual/`](manual/) | manual de uso | tarefa da pessoa usuária | [README](manual/README.md) |

## Specs

| Spec | Capacidade |
|---|---|
| [event-normalization](specs/event-normalization.md) | Traduzir os eventos dos três CLIs num vocabulário único de "momentos". **É a fonte da verdade do mapa de eventos** |
| [capture-transports](specs/capture-transports.md) | Receber os eventos em qualquer terminal: hooks, `notify`, PTY, `stream-json`, ACP |
| [speech-output](specs/speech-output.md) | Transformar momentos em fala sem virar ruído |
| [speech-input](specs/speech-input.md) | Responder por voz, nas quatro formas |
| [configuration](specs/configuration.md) | Camadas, precedência e o conjunto de chaves |
| [interfaces](specs/interfaces.md) | CLI e GUI, com paridade real |
| [portability](specs/portability.md) | Linux, macOS e Windows de verdade |

As regras que valem nas quatro árvores estão em [`../README.md`](../README.md).
