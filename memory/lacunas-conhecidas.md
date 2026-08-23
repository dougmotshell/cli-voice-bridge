# Lacunas conhecidas

O que **não** funciona, em 2026-08-23. Existe para que ninguém descubra por
tentativa e erro o que já se sabe, e para que nada disto passe por defeito novo.

Cada uma tem um documento que explica o porquê e o que falta decidir; aqui fica
só o resumo e o ponteiro.

| Lacuna | Sintoma para quem usa | Onde está descrita |
|---|---|---|
| **Detecção de presença** | `falar = "ausente"` cala em vez de falar | [presence-detection](../docs/pt-BR/specs/presence-detection.md) |
| **Encerramento ordenado** | Daemon morto por sinal deixa o socket no disco e pode deixar áudio tocando | [daemon-lifecycle](../docs/pt-BR/specs/daemon-lifecycle.md) |
| **Teto do cache de áudio** | O diretório `cache-audio/` só cresce | [speech-output](../docs/pt-BR/specs/speech-output.md) |
| **Supervisão do sidecar** | Sidecar morre e ninguém levanta; a voz vira a do sistema | [daemon-lifecycle](../docs/pt-BR/specs/daemon-lifecycle.md) |
| **Entrada por voz** | Não dá para responder falando; nada de `cvb listen` | [speech-input](../docs/pt-BR/specs/speech-input.md) |
| **Wrapper PTY** | Sem narração contínua e sem injeção de texto | [capture-transports](../docs/pt-BR/specs/capture-transports.md) |
| **GUI** | Não existe | [interfaces](../docs/pt-BR/specs/interfaces.md) |
| **Resumo de mensagem longa** | Mensagem do assistente é cortada por número de caracteres, sem resumir | [speech-output](../docs/pt-BR/specs/speech-output.md) |
| **IPC no Windows** | O daemon não sobe: named pipes não implementados | [portability](../docs/pt-BR/specs/portability.md) |

## O que aprender com o padrão delas

**Três destas lacunas têm a mesma raiz:** presença, encerramento ordenado e
supervisão do sidecar são todas coisas que o sistema operacional faz melhor que o
projeto. Presença é ociosidade que o SO já mede; encerramento é manipulação de
sinal; supervisão é systemd ou launchd. A tentação de implementar cada uma à mão
é o que se deve resistir.

**Duas são invisíveis até doerem:** o cache sem teto e a retenção de log que
nunca poda. Nenhuma incomoda no primeiro mês. É por isso que estão escritas.

**A do Windows é a única que impede o projeto de rodar.** As outras degradam.
Se um dia houver uma máquina Windows para testar, essa é a primeira.
