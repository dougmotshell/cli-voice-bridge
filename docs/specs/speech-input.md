# Spec — Entrada de voz

**Capacidade:** responder ao agente falando, em todas as formas em que isso faz
sentido — de "sim" a um prompt ditado inteiro.

**ADRs que restringem este spec:** [ADR-0005](../decisions/0005-wrapper-pty-como-transporte-complementar.md),
[ADR-0006](../decisions/0006-stt-offline-na-maquina.md).
**Nível C4:** [componente](../architecture/03-component.md) — `hookd::listen`.

## Problema

Falar é fácil; entregar o que foi falado ao processo certo é que não é. O CLI
está numa TUI interativa, possivelmente num terminal integrado de IDE, e não tem
API de entrada. E reconhecimento errado num "sim" pode autorizar um `rm -rf`.

## Escopo

Dentro: captura de microfone, detecção de fala, transcrição offline, os quatro
modos de resposta, e a política de confirmação.
Fora: a síntese ([speech-output](speech-output.md)) e o transporte dos eventos
([capture-transports](capture-transports.md)).

## Os quatro modos

Todos implementados; a configuração escolhe quais ficam ativos, por CLI.

### 1. Resposta fechada — a mais segura

Quando o momento é `decision.needed`, o daemon fala a pergunta e abre uma janela
de escuta com **vocabulário restrito**: sim, não, sempre, nunca, opção um/dois/três,
cancelar. Vocabulário fechado é quase infalível, e a decisão volta pelo próprio
hook — o `PermissionRequest` do Claude e do Copilot aceita
`decision`/`behavior` na resposta, e o `PreToolUse` aceita
`permissionDecision`. Não passa por teclado nenhum.

**Regra de segurança.** Comando classificado como destrutivo (`rm -rf`, `git push
--force`, `DROP`, qualquer coisa que a configuração marque) **nunca** é
autorizado só por voz: o daemon pede repetição de uma palavra-código, ou recusa e
manda decidir na tela. Voz é o canal com maior taxa de erro do sistema; a
autorização perigosa não anda por ele sozinha.

Fora da janela de escuta, nada é ouvido. O microfone não fica aberto.

### 2. Ditado para a área de transferência

Atalho global grava, transcreve, coloca o texto no clipboard e avisa. A pessoa
cola e confere antes de enviar. Funciona com qualquer CLI, em qualquer terminal,
sem wrapper e sem permissão especial. É o modo padrão porque é o que nunca
quebra.

### 3. Ditado injetado no CLI

Com o wrapper PTY ativo (`cvb wrap -- claude`), o texto transcrito é escrito
direto no `stdin` do CLI. Sem simular teclado, sem depender de foco de janela,
funciona igual no terminal do sistema e no integrado do VS Code ou do IntelliJ.

Duas variantes, configuráveis: **escrever e parar** (a pessoa revisa e aperta
Enter) ou **escrever e enviar**. A segunda é o mais fluido e o mais arriscado;
padrão é a primeira.

Sem wrapper, há o recurso de simulação de teclado no SO — precisa de permissão de
Acessibilidade no macOS e é frágil no Wayland. Fica disponível, marcado como o
caminho de menor confiança.

### 4. Conversa por protocolo

No modo `cvb console --cli <nome>` o projeto é o cliente ACP/app-server: a fala
vira mensagem de protocolo, sem teclado nem PTY. É o caminho mais limpo e o que
custa a TUI original. Ver [capture-transports](capture-transports.md).

## Cadeia de reconhecimento

```
microfone → captura → VAD → recorte da fala → STT → pós-processamento → destino
```

- **Captura.** Taxa e canais fixos na entrada do modelo; o dispositivo é
  escolhido na configuração e conferido pelo `cvb doctor`.
- **VAD.** Detecta início e fim de fala para não transcrever silêncio nem cortar
  a pessoa no meio. Silero VAD é a hipótese de trabalho.
- **STT.** Offline, na máquina, pt-BR e en-US ([ADR-0006](../decisions/0006-stt-offline-na-maquina.md)).
  TODO: escolher entre `whisper.cpp` (multilíngue, qualidade alta, mais pesado) e
  `sherpa-onnx` (mais leve, streaming de verdade). Medir na máquina alvo antes de
  decidir — o `voice-clone` já mostrou que medir muda a resposta.
- **Pós-processamento.** No modo fechado, casa contra o vocabulário e devolve
  confiança; abaixo do limiar, não decide — pergunta de novo ou devolve à tela.
  No ditado, aplica um dicionário de termos técnicos que o modelo erra
  (nomes de ferramenta, siglas do projeto), configurável.

## Acionamento

- **Push-to-talk** com atalho global: grava enquanto a tecla estiver pressionada.
  É o padrão — sem microfone aberto, sem escuta acidental.
- **Janela de escuta automática** após um momento crítico, com duração limitada.
- **Palavra de ativação:** TODO: decidir se entra. Exige microfone sempre aberto,
  o que contraria o princípio de escuta mínima. Provavelmente não.

## Privacidade

- O microfone só abre por ação explícita ou dentro de uma janela de escuta
  anunciada. A GUI e a CLI mostram, de forma inconfundível, quando está gravando.
- Áudio de microfone é **descartado** logo após a transcrição. Não há gravação
  persistente sem a pessoa ligar explicitamente, e mesmo então fica fora do git.
- Transcrição não sai da máquina.
- Voz é dado biométrico: vale aqui o mesmo regime do `voice-clone`.

## Alternativas consideradas

**STT de nuvem.** Descartado: manda a voz e o conteúdo do trabalho para fora,
contra o princípio central do projeto.

**Só ditado livre, sem modo fechado.** Descartado: a resposta mais frequente é
"sim" ou "não" numa pergunta de permissão, e é justamente aí que erro custa caro.

**Só modo fechado.** Descartado: a pessoa pediu todas as formas, e ditar um
prompt longo é metade do valor.

## Plano de teste

TODO: escrever. Mínimo: bateria de áudios gravados de "sim"/"não"/"cancelar" em
pt-BR com ruído, afirmando classificação e confiança; teste de que comando
destrutivo nunca é autorizado por um único "sim"; e teste de injeção no PTY com
um programa de eco no lugar do CLI.

## Questões em aberto

- Motor de STT (acima).
- Palavra de ativação (acima).
- Como sinalizar "estou gravando" quando não há GUI aberta e o terminal está em
  outra janela. Provável: um som curto de início e fim, mais o indicador da GUI.
