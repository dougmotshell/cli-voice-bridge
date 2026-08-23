# Arquitetura — modelo C4

Um arquivo por nível. Cada documento fica no seu nível: detalhe de contêiner não
entra no de contexto.

| Nível | Documento | Responde |
|---|---|---|
| 1 | [01-context.md](01-context.md) | Quem usa, com que sistemas conversa, que restrições atravessam tudo |
| 2 | [02-container.md](02-container.md) | Que peças executáveis existem e como se comunicam |
| 3 | [03-component.md](03-component.md) | O que existe dentro do `hookd` |
| 4 | — | Ausente de propósito: não há código ainda, e o tamanho do projeto não justifica |

Diagramas em Mermaid dentro do Markdown, para diferenciarem e serem revisáveis.
