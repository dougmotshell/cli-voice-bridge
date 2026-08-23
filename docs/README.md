# Documentação do cli-voice-bridge

A documentação existe em **pt-BR e en-US**, sempre. `docs/` é dividido por
idioma; um `docs/` plano nunca está certo.

| Idioma | Árvore | Papel |
|---|---|---|
| pt-BR | [`pt-BR/`](pt-BR/) | **Fonte da verdade.** É aqui que se escreve primeiro |
| en-US | [`en-US/`](en-US/) | Tradução. Cada arquivo abre apontando para o original em pt-BR |

Os nomes de arquivo são **idênticos** nas duas subárvores, para que uma tradução
seja sempre um irmão e nunca uma bifurcação. Arquivo que ainda falta numa língua
recebe um `TODO:` explícito — nunca some em silêncio.

## As quatro árvores, dentro de cada idioma

| Árvore | Padrão | Um arquivo por |
|---|---|---|
| `architecture/` | C4 | nível |
| `specs/` | SDD | capacidade |
| `decisions/` | ADR (MADR) | decisão, `NNNN-titulo-em-kebab.md` |
| `manual/` | manual de uso | tarefa da pessoa usuária |

Índices por idioma: [pt-BR](pt-BR/README.md) · [en-US](en-US/README.md).

## Regras que valem nas duas línguas e nas quatro árvores

- **Diagramas são texto** (Mermaid dentro do Markdown), para diferenciarem.
- **Cruze as referências nos dois sentidos:** todo spec nomeia os ADRs que o
  restringem; todo ADR nomeia o nível C4 e os specs que ele move.
- **ADR é append-only.** ADR aceito é substituído por um novo, nunca reescrito.
- **Nada de invenção.** O que o projeto ainda não decidiu fica como `TODO:`.

## O que está deliberadamente ausente

- **`architecture/04-code.md`** — o tamanho do projeto não justifica um diagrama
  de classes. Se um módulo ficar intrincado o bastante, cria-se o arquivo só
  para ele.
- **Documentação de API.** Não há API pública. O protocolo de IPC é interno e
  está descrito em `specs/interfaces.md`.
