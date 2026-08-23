# Documentação do cli-voice-bridge

Toda documentação deste projeto mora numa destas quatro árvores. Nunca solta na
raiz do repositório, nunca dois padrões no mesmo arquivo.

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

## O que está deliberadamente ausente

- **`docs/architecture/04-code.md`** — não há código ainda, e o tamanho do
  projeto não justifica um diagrama de classes. Se um módulo ficar intrincado o
  bastante, cria-se o arquivo só para ele.
- **Uma segunda língua.** O projeto é configurado só em pt-BR, então as árvores
  são planas. Se um dia entrar en-US, cada árvore ganha subdiretório por língua
  com **os mesmos nomes de arquivo**, e a tradução aponta para o original.
- **Documentação de API.** Não há API pública. O protocolo de IPC é interno e
  está descrito em [interfaces](specs/interfaces.md).

## Regras que valem nas quatro árvores

- **Diagramas são texto** (Mermaid dentro do Markdown), para diferenciarem.
- **Cruze as referências nos dois sentidos:** todo spec nomeia os ADRs que o
  restringem; todo ADR nomeia o nível C4 e os specs que ele move.
- **Nada de invenção.** O que o projeto ainda não decidiu fica como `TODO:`.
