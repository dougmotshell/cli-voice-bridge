# Spec — Ciclo de vida do daemon

**Capacidade:** o `hookd` subir quando precisa, morrer sem deixar sujeira, e
manter o sidecar de pé.

**ADRs que restringem este spec:** [ADR-0001](../decisions/0001-nucleo-em-rust-com-cliente-de-hook-separado.md),
[ADR-0003](../decisions/0003-tts-delegado-ao-voice-clone.md),
[ADR-0008](../decisions/0008-ipc-por-socket-local.md).
**Nível C4:** [contêiner](../architecture/02-container.md).

## Estado

| Parte | Estado |
|---|---|
| Instância única | **funciona** — `Ouvinte::abrir` recusa se já houver alguém atendendo |
| Socket órfão de execução anterior | **funciona** — é detectado e removido no arranque |
| Encerramento ordenado | **não existe** — o daemon morre por sinal, sem drenar nem limpar |
| Arranque sob demanda | **não existe** — o `hookc` sai em silêncio se não achar o daemon |
| Autostart no login | **não existe** |
| Supervisão do sidecar | **não existe** — morreu, ninguém levanta |

## Problema

Um daemon que só a pessoa sobe à mão só serve enquanto ela lembra de subir. E um
daemon que morre por sinal deixa três coisas para trás: o socket no disco, a
fila sem drenar, e um processo de reprodutor de áudio possivelmente tocando.

Nada disso é grave hoje — o arranque seguinte limpa o socket, a fila é volátil por
natureza e o reprodutor termina sozinho. Mas cada um vira defeito real assim que
algo depender de encerramento previsível.

## Escopo

Dentro: arranque, encerramento, instância única, socket órfão, e supervisão do
sidecar de síntese.
Fora: o que o daemon faz enquanto vive — isso está nos outros specs.

## Design

### Arranque

Três caminhos, do mais simples ao mais automático:

1. **À mão:** `cvb daemon start`. É o que existe hoje, na forma de rodar o
   binário. TODO: o subcomando ainda sai com "não implementado".
2. **Sob demanda:** o primeiro `hookc` que não encontrar o socket pede para
   subir. TODO: decidir se vale — tem o risco de vários hooks tentarem subir ao
   mesmo tempo, o que a recusa de instância única já resolve, mas com um custo
   de processos nascendo e morrendo.
3. **No login**, conforme configuração:

| Sistema | Mecanismo |
|---|---|
| Linux | unidade systemd de usuário (`~/.config/systemd/user/`) |
| macOS | `launchd` LaunchAgent (`~/Library/LaunchAgents/`) |
| Windows | Tarefa Agendada no logon, ou a chave `Run` |

TODO: `cvb daemon install-autostart` / `uninstall-autostart`, com o mesmo
`--dry-run` do `cvb install`. Vale a mesma regra: nunca sobrescrever unidade
alheia.

### Encerramento ordenado

Ao receber `SIGTERM` ou `SIGINT` (e o equivalente no Windows):

1. Parar de aceitar conexões novas.
2. **Matar o reprodutor em curso.** Deixar um áudio tocando depois de o daemon
   morrer é o pior dos mundos: ninguém consegue cortá-lo.
3. Descartar a fila. Não drenar: se a pessoa está encerrando, ela não quer ouvir
   mais nada. O que era crítico já perdeu o sentido — o CLI que esperava também
   vai morrer.
4. Fechar a conexão com o sidecar sem matá-lo: ele é outro processo, com dono
   próprio.
5. Remover o socket.

`Fila::encerrar` já existe e hoje só os testes usam; é aqui que ele entra.

**Por que o `Drop` do `Ouvinte` não basta:** morte por sinal não desenrola a
pilha, então o `Drop` não roda e o socket fica. Já é tratado defensivamente no
arranque seguinte, mas tratar na saída é mais limpo e deixa a máquina sem
resíduo.

### Instância única

`Ouvinte::abrir` tenta conectar no endereço antes de escutar. Se alguém atende, é
`AddrInUse` e o novo daemon desiste; se ninguém atende, o socket é lixo de uma
execução que morreu, e é removido. Isso já funciona.

### Supervisão do sidecar

O sidecar é um processo Python separado, com dono próprio (ADR-0003). O daemon
não o inicia hoje.

TODO: decidir entre três posturas, que têm consequências bem diferentes:

- **Não supervisionar** (hoje). O daemon cai para a voz do sistema e avisa. É
  honesto e simples, e a pessoa precisa levantar o sidecar à mão.
- **Reiniciar quando morrer.** Precisa de limite de tentativas e recuo, senão um
  sidecar que quebra na carga do modelo vira um laço de processos.
- **Deixar o supervisor do sistema cuidar** — systemd/launchd, com o mesmo
  mecanismo do autostart. Provavelmente a resposta certa: é onde essa
  responsabilidade já mora.

## Dados e contratos

Nenhum estado do daemon é persistido. Sessões vivas, fila e silenciamento são
todos voláteis, e reiniciar zera tudo — de propósito: o que importa é o que está
acontecendo agora.

## Privacidade

O log de sessão tem retenção configurável (`privacidade.retencao_log_dias`).
TODO: a poda por retenção ainda não foi implementada; o encerramento ordenado é
um bom lugar para ela rodar.

## Alternativas consideradas

**Drenar a fila antes de sair** em vez de descartar. Descartado: encerramento que
demora é encerramento que a pessoa mata com `-9`, e aí não houve encerramento
ordenado nenhum.

**Um arquivo de PID** para instância única. Descartado: o socket já responde à
pergunta "tem alguém vivo?" com menos partes móveis e sem PID obsoleto.

**O daemon iniciar e monitorar o sidecar como filho.** Considerado; amarra o
ciclo de vida dos dois e faz o daemon herdar os problemas de arranque do Python.
Fica atrás de deixar o supervisor do sistema cuidar.

## Plano de teste

TODO: escrever. Mínimo: um teste que sobe dois daemons no mesmo endereço e afirma
que o segundo recusa; um que deixa um socket órfão e afirma que o arranque o
remove; e um teste de encerramento que afirma que o socket some e que nenhum
processo de reprodutor sobrevive.

## Questões em aberto

- Arranque sob demanda vale a pena (acima).
- Postura de supervisão do sidecar (acima).
- Se o `cvb daemon start` deve fazer *fork* e devolver o controle, ou ficar em
  primeiro plano e deixar o desacoplamento por conta de quem chama.
