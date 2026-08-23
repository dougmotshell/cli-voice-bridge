# Uso no dia a dia

TODO: escrever de verdade quando o projeto rodar. O esqueleto abaixo é o
comportamento pretendido.

## O básico

Abra os CLIs como você já abre. Com os hooks instalados, o `cli-voice-bridge`
avisa por voz quando o agente precisa de você: pedido de permissão, pergunta,
turno concluído, subagente iniciado ou terminado.

Não muda nada no seu jeito de trabalhar — salvo se você quiser narração ou
ditado injetado, que precisam do wrapper.

## Responder por voz

**Pergunta fechada.** Quando o agente pede permissão, ele fala a pergunta e abre
uma janela de escuta. Responda "sim", "não", "sempre", "nunca", "cancelar" ou
"opção um/dois/três". A resposta volta pelo hook, sem passar por teclado.

Comando destrutivo — `rm -rf`, `git push --force` e o que estiver na sua lista —
**nunca** é autorizado por um único "sim". Ele pede uma confirmação a mais ou
manda você decidir na tela. É de propósito: voz é o canal com maior taxa de erro.

**Ditar um prompt.** Segure o atalho global, fale, solte. O texto transcrito vai
para a área de transferência e você cola — é o modo padrão, porque funciona em
qualquer terminal e nunca quebra.

**Ditar direto no CLI.** Abra o CLI pelo wrapper:

```bash
cvb wrap -- claude
```

Aí o texto ditado é escrito direto na entrada do CLI. Funciona igual no terminal
do sistema e no integrado do VS Code ou do IntelliJ. Por padrão ele escreve e
para, para você revisar antes de enviar.

## Silenciar

```bash
cvb mute 30m      # cala por meia hora
cvb unmute
```

Na GUI, pelo ícone de bandeja. Trocar para o perfil `reuniao` é o equivalente
mais fino: cala tudo menos o crítico, e o crítico sai sem dizer o conteúdo.

## Ver o que está acontecendo

```bash
cvb events --follow      # o fluxo de momentos, ao vivo
cvb daemon status
```

A GUI mostra o mesmo em forma de painel, com o que está falando agora, o que
está na fila e um botão de cortar a fala.

## Quando você volta a digitar

Se você submete um prompt enquanto ele está falando, ele cala na hora. Você já
voltou; não precisa mais do aviso.
