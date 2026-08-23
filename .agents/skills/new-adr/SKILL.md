<!-- managed-by:cli-voice-bridge/sync-ai-surfaces — do not edit by hand -->
---
name: new-adr
description: Cria um novo ADR em docs/pt-BR/decisions/ no formato MADR, com o próximo número livre, e atualiza o índice. Use ao registrar uma decisão de arquitetura do cli-voice-bridge.
---
<!-- fonte: skills/new-adr/SKILL.md -->

# Novo ADR

Registra uma decisão de arquitetura. Um arquivo, uma decisão, nunca reescrito.

## Passos

1. **Confira que é mesmo uma decisão de arquitetura.** ADR é para escolha que
   restringe o futuro e cuja alternativa era defensável. Escolha de nome de
   variável não é ADR; escolher socket local em vez de porta TCP é.

2. **Descubra o próximo número.** `ls docs/pt-BR/decisions/` e some um ao maior.
   Número nunca se reaproveita, nem de ADR abandonado.

3. **Copie `templates/adr.md`** para
   `docs/pt-BR/decisions/NNNN-titulo-em-kebab-case.md`. Título em en-US no nome do
   arquivo? Não: aqui os nomes de ADR são descritivos em pt-BR sem acento, como
   os que já existem — siga o padrão do diretório.

4. **Preencha, sem inventar.** O que você não sabe vira `TODO:`, não chute.
   - **Contexto** é o que era verdade quando se decidiu, incluindo medições e
     restrições. Quem ler daqui a um ano precisa entender a pressão do momento.
   - **Decisão** no presente, afirmativa.
   - **Consequências** incluem as ruins e o que a decisão passa a proibir.
   - **Alternativas** precisam ser reais e ter o motivo do descarte. Este é o
     campo que dá valor ao ADR.

5. **Cruze as referências nos dois sentidos.** O ADR nomeia o nível C4 que move e
   os specs que restringe; volte nesses specs e acrescente o ADR na lista deles.
   Referência de mão única apodrece.

6. **Atualize a tabela** em `docs/pt-BR/decisions/README.md`.

## Se a decisão substitui outra

Não edite o ADR antigo além de uma linha: `Status: substituído por NNNN`. O
histórico fica. O ADR novo explica o que mudou desde o antigo — normalmente uma
medição nova ou uma restrição que caiu.
