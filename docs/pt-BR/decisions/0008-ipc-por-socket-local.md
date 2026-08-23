# ADR-0008 — IPC por socket local, nunca por porta TCP

**Status:** aceito — 2026-08-23
**Nível C4:** [contêiner](../architecture/02-container.md)
**Specs que esta decisão move:** [capture-transports](../specs/capture-transports.md), [interfaces](../specs/interfaces.md), [portability](../specs/portability.md)

## Contexto

Quatro tipos de cliente falam com o `hookd`: o `hookc` (centenas de vezes por
sessão, precisa ser rápido), a CLI, a GUI e o sidecar de TTS. O que trafega é o
conteúdo do trabalho — caminho, comando, trecho de mensagem do assistente.

## Decisão

Socket UNIX no Linux e no macOS; named pipe no Windows. Nunca porta TCP, nem em
`localhost`.

O protocolo é orientado a mensagem, com versão no handshake. Cliente de versão
incompatível recebe recusa explicativa em vez de comportamento indefinido.

## Consequências

**Boas.** A permissão do sistema de arquivos já faz o controle de acesso — sem
inventar autenticação. Nada escuta na rede, então nada é alcançável de fora nem
aparece numa varredura de portas. Conexão local é mais rápida que a pilha TCP,
o que importa no caminho do `hookc`.

**Ruins.** Dois caminhos de código por causa do Windows. E o daemon fica preso à
máquina: um CLI rodando por `ssh` noutra máquina não alcança o daemon local — é a
questão em aberto de [capture-transports](../specs/capture-transports.md).

**Restringe.** Nenhum recurso pode assumir acesso remoto. Se um dia for
necessário, é um transporte novo, explícito, com ADR próprio e autenticação de
verdade — não relaxar este.

## Alternativas

**HTTP em `localhost`.** Descartado: qualquer processo do usuário alcança, exige
inventar autenticação, e o `voice-clone` já tomou a decisão análoga de escutar só
em `127.0.0.1` justamente por não querer superfície de rede.

**Arquivo de fila em disco.** Descartado: latência e limpeza de sujeira, e deixa
o conteúdo do trabalho parado em disco.

**D-Bus.** Descartado: só Linux.
