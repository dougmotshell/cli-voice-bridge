# Spec — Detecção de presença

**Capacidade:** saber se a pessoa está por perto e atenta, para falar só quando
a fala acrescenta alguma coisa.

**ADRs que restringem este spec:** [ADR-0008](../decisions/0008-ipc-por-socket-local.md).
**Nível C4:** [componente](../architecture/03-component.md) — módulo `policy::presenca`.

## Estado

**Não implementada.** É a única parte da política de voz que não vale hoje:
`falar = "ausente"` está calando em vez de falar, que é o fallback documentado em
[speech-output](speech-output.md) ("assumir presente, falar menos"). Enquanto
este spec não virar código, quem quiser ouvir um momento de urgência média
precisa marcá-lo explicitamente como `"sempre"`.

## Problema

`falar = "ausente"` é o ajuste que mais reduz ruído: fala só quando a pessoa não
está olhando. Se ela está com o terminal na frente e acabou de digitar, já viu o
pedido de permissão na tela — ouvir a mesma coisa em voz alta é redundância, e
redundância é o que faz alguém desligar o projeto.

O problema é que "não está olhando" tem duas leituras muito diferentes de custo, e
escolher a errada trava o recurso numa plataforma inteira.

## Escopo

Dentro: decidir presente/ausente e entregar isso à política de voz.
Fora: o que fazer com a resposta — isso é [speech-output](speech-output.md). Fora
também: detectar que a pessoa voltou **depois** de a fala começar; isso já existe
e vem por evento (`user.returned`), não por sensor.

## As duas leituras de "ausente"

| Leitura | O que mede | Custo |
|---|---|---|
| **Ociosa na máquina** | Tempo desde a última tecla ou movimento de mouse, em qualquer lugar | Baixo e uniforme nos três sistemas |
| **Olhando para aquela janela** | Qual janela tem foco, e se é o terminal daquela sessão do CLI | Alto, e impossível no Wayland sem cooperação do compositor |

**A recomendação é a primeira.** Não porque a segunda seja pior — ela é melhor —
mas porque a segunda esbarra num problema que nenhum esforço de implementação
resolve: **o daemon não sabe em que janela aquele CLI está.** O payload do hook
traz `session_id`, `cwd` e `permission_mode`; não traz identificador de janela, e
não há como correlacionar de forma confiável um processo de CLI com uma janela de
terminal, muito menos com uma aba de terminal integrado de IDE.

Ou seja: mesmo com foco de janela resolvido no Linux e no macOS, ainda faltaria a
metade difícil. Ociosidade não tem esse problema — é uma pergunta sobre a pessoa,
não sobre a topologia de janelas.

## Design

```
policy::presenca::estado() -> Presenca { Presente, Ausente, Desconhecida }
```

`Desconhecida` é uma resposta legítima e não um erro: no Wayland sem portal, é o
que se tem. Quem consome trata `Desconhecida` como `Presente` — assumir presente
faz falar menos, e silêncio incomoda menos que ruído.

Limiar padrão: **60 segundos** sem entrada. TODO: medir; 60 s é chute informado,
não medição.

### Como obter ociosidade em cada sistema

| Sistema | Caminho | Observação |
|---|---|---|
| Linux/X11 | extensão `XScreenSaver` (`XScreenSaverQueryInfo` → `idle`) | Confiável e barato |
| Linux/Wayland | `org.freedesktop.ScreenSaver` no D-Bus, ou o protocolo `ext-idle-notify-v1` | Depende do compositor; GNOME e KDE expõem, outros não |
| macOS | `CGEventSourceSecondsSinceLastEventType` com `kCGAnyInputEventType` | Sem permissão especial |
| Windows | `GetLastInputInfo` | Sem permissão especial |

Nenhum desses exige permissão de Acessibilidade nem grava conteúdo — só a
quantidade de segundos desde o último evento de entrada. Ver *Privacidade*.

TODO: decidir se entra dependência nativa por sistema ou se o daemon delega a um
comando externo, como fez o [ADR-0009](../decisions/0009-reproducao-por-reprodutor-do-sistema.md)
para reprodução. A analogia é tentadora, mas aqui a consulta é frequente e um
processo por consulta não se pagaria.

## Dados e contratos

O estado é consultado no momento de decidir a fala, não assinado. Nada é
guardado: nem histórico de ociosidade, nem carimbo de quando a pessoa esteve
presente.

## Privacidade

Ociosidade é um número em segundos. **Não** se lê qual tecla foi apertada, qual
janela está em foco, qual aplicativo está aberto nem qual o título da janela —
mesmo onde o sistema permitiria. É informação que não é necessária para a decisão
e que, uma vez lida, viraria mais uma coisa a proteger.

Nada disso sai da máquina, como o resto do projeto.

## Alternativas consideradas

**Foco de janela em vez de ociosidade.** Melhor sinal, e descartado pelo motivo
acima: o daemon não sabe qual janela é a daquele CLI, então o sinal melhor não
seria aplicável de qualquer forma.

**Perguntar ao próprio CLI.** Nenhum dos três expõe "a pessoa está olhando para
mim". O que existe é `Notification` com `idle_prompt` no Claude, que diz que o
*agente* está ocioso — informação oposta.

**Deixar a pessoa alternar à mão** (`cvb away` / `cvb back`). Descartado como
solução: ninguém lembra de avisar que saiu, e o valor está justamente em não
precisar avisar. Continua útil como complemento — TODO: avaliar junto dos perfis.

**Não implementar e deixar `ausente` como sinônimo de `nunca`.** É o que vale
hoje, e é honesto enquanto está escrito. Não serve como decisão final: some com
metade da utilidade da configuração.

## Plano de teste

TODO: escrever. O mínimo: uma abstração de relógio de ociosidade injetável, para
testar a política sem depender do sistema; e um teste que afirme que
`Desconhecida` se comporta como `Presente`. A parte específica de cada sistema
não tem como ser testada em CI sem uma sessão gráfica — vai para a lista de
verificação manual do manual.

## Questões em aberto

- Limiar de ociosidade (acima).
- Dependência nativa por sistema ou comando externo (acima).
- O que fazer quando a pessoa está presente mas noutro monitor, de costas para o
  terminal. Provável resposta: nada — é o limite do que ociosidade mede, e
  aceitar o limite é melhor que fingir precisão que não existe.
