<!-- managed-by:cli-voice-bridge/sync-ai-surfaces — do not edit by hand -->
---
applyTo: "docs/decisions/*.md"
---
<!-- fonte: .claude/rules/adr.md -->

# ADRs

Formato MADR, template em [`templates/adr.md`](../../templates/adr.md).

**Append-only.** ADR aceito nunca é reescrito para mudar a decisão. Se a decisão
mudou, escreva um ADR novo e marque o antigo com
`Status: substituído por NNNN`. Número não se reaproveita, nem depois de um ADR
ser abandonado.

Correção de erro factual dentro de um ADR aceito é permitida, mas escreva a
revisão **dentro dele**, datada, em vez de apagar o que estava.

**Todo ADR nomeia o nível C4 que ele move e os specs que ele restringe**, com
link. E o spec correspondente nomeia o ADR de volta — a referência é nos dois
sentidos, senão uma das pontas apodrece sem ninguém notar.

**As alternativas são obrigatórias e precisam ser reais.** "Não fazer nada" e uma
opção que ninguém considerou não contam. O valor de um ADR está em registrar o
que foi descartado e por quê — é isso que impede a discussão de voltar em seis
meses.

**Consequências incluem as ruins.** ADR que só lista vantagens não foi pensado.

Depois de criar ou renumerar, atualize a tabela em
[`docs/decisions/README.md`](../../docs/decisions/README.md).
