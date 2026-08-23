<!-- managed-by:cli-voice-bridge/sync-ai-surfaces — do not edit by hand -->
---
name: new-spec
description: Cria um novo spec de capacidade em docs/pt-BR/specs/ no formato SDD, com escopo, requisitos, design, alternativas, plano de teste e questões em aberto. Use ao desenhar uma capacidade nova do cli-voice-bridge.
---
<!-- fonte: skills/new-spec/SKILL.md -->

# Novo spec

Um arquivo por **capacidade** — não por módulo, não por arquivo de código. Se
você não consegue dizer a capacidade em uma frase que faça sentido para quem usa,
provavelmente não é um spec: é detalhe de implementação.

## Passos

1. **Copie `templates/spec.md`** para `docs/pt-BR/specs/<nome-em-kebab>.md`. Nome em
   en-US, como os existentes (`speech-output`, `event-normalization`).

2. **Comece pelo problema**, não pela solução. Uma capacidade cujo problema você
   não consegue escrever em três frases ainda não está pronta para spec.

3. **Escreva o "Fora do escopo" com a mesma seriedade que o "Dentro".** É a parte
   que evita o spec crescer sem controle, e é onde se aponta qual outro spec
   cuida do que ficou de fora.

4. **Alternativas consideradas são obrigatórias**, com o motivo do descarte.

5. **Plano de teste.** Se você ainda não sabe como testar, escreva `TODO:` e diga
   o que precisaria existir para saber. Um spec sem plano de teste vira código
   sem teste.

6. **Questões em aberto ficam no fim, nomeadas.** "Falta decidir X" é informação
   útil; silêncio sobre X é dívida escondida.

7. **Cruze as referências:** o spec lista os ADRs que o restringem e o nível C4
   correspondente; volte nesses ADRs e acrescente o spec na linha *Specs que esta
   decisão move*.

## O que não vai num spec

Decisão de arquitetura com alternativas descartadas — isso é ADR
(`/new-adr`). O spec **cita** o ADR; não repete o raciocínio dele.
