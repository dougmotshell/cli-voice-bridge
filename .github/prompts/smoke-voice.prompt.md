<!-- managed-by:cli-voice-bridge/sync-ai-surfaces — do not edit by hand -->
---
name: smoke-voice
description: Teste de fumaça da cadeia de voz do cli-voice-bridge, de ponta a ponta — daemon, evento, política, síntese, reprodução, captura e transcrição. Use antes de dar qualquer mudança de áudio por concluída.
agent: agent
---
<!-- fonte: skills/smoke-voice/SKILL.md -->

# Teste de fumaça da voz

Não existe teste automatizado que julgue se uma voz soa bem. Mudança em áudio só
está feita depois de ouvida.

## Saída

1. `cvb doctor` — tem de passar limpo. Se acusa algo, pare e resolva antes.
2. `cvb say "teste de voz do cli-voice-bridge"` — ouça. A voz é a certa? Soa
   natural? Quanto tempo entre o comando e o primeiro som?
3. **Dispare um momento de verdade**, não um simulado: abra um CLI de IA e
   provoque um pedido de permissão. `cvb events --follow` numa janela ao lado
   mostra o momento chegando.
4. **Corte:** com ele falando algo longo, provoque um momento crítico. A fala
   anterior tem de ser cortada, não enfileirada.
5. **Fallback:** derrube o sidecar e repita o passo 2. Tem de falar com a voz do
   sistema e avisar — nunca ficar mudo.

## Entrada

6. `cvb listen` — fale uma frase técnica em pt-BR e confira a transcrição.
7. **Pergunta fechada:** provoque um pedido de permissão e responda "sim" por
   voz. Confira que a decisão chegou ao CLI.
8. **Recusa de destrutivo:** provoque um pedido para um comando destrutivo e
   responda "sim". Ele **não** pode autorizar com um único sim.
9. **Ditado:** segure o atalho, dite uma frase, confira o destino configurado
   (área de transferência ou entrada do CLI).

## O que relatar

Diga o que você ouviu e mediu, não o que esperava ouvir. Latência do primeiro som
e naturalidade são as duas coisas que só o ouvido julga. Se algum passo não pôde
ser executado, diga qual e por quê — teste de fumaça pela metade relatado como
completo é pior que nenhum.
