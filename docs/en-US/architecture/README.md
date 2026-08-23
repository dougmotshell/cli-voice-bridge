# Architecture — the C4 model

> Translation of [`../../pt-BR/architecture/README.md`](../../pt-BR/architecture/README.md),
> which is the source of truth.

One file per level. Each document stays at its own level: container detail does
not belong in the context document.

| Level | Document | Answers |
|---|---|---|
| 1 | [01-context.md](01-context.md) | Who uses it, what systems it talks to, what constraints cut across everything |
| 2 | [02-container.md](02-container.md) | What executable pieces exist and how they communicate |
| 3 | [03-component.md](03-component.md) | What lives inside `hookd` |
| 4 | — | Deliberately absent: the size of the project does not justify a class diagram |

Diagrams are Mermaid inside the Markdown, so they diff and can be reviewed.
