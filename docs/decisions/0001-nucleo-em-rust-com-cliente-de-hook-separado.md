# ADR-0001 — Núcleo em Rust, com o cliente de hook separado do daemon

**Status:** aceito — 2026-08-23
**Nível C4:** [contêiner](../architecture/02-container.md)
**Specs que esta decisão move:** [capture-transports](../specs/capture-transports.md), [portability](../specs/portability.md)

## Contexto

Os hooks dos três CLIs executam um comando **em série com o agente**: enquanto o
hook não retorna, o agente espera. `PreToolUse` dispara a cada chamada de
ferramenta — centenas de vezes numa sessão de trabalho. Qualquer custo fixo de
arranque aparece multiplicado, e aparece como lentidão da ferramenta de IA, não
como lentidão deste projeto.

Ao mesmo tempo, o trabalho de verdade — carregar modelo de STT, manter fila de
fala, conversar com o sidecar do XTTS — é caro e precisa de estado entre eventos.
Não cabe num processo que nasce e morre a cada hook.

A escolha do interpretador foi medida no `voice-clone`: o arranque do Python com
os imports do projeto está na casa das dezenas de milissegundos, antes de
qualquer trabalho útil.

## Decisão

Duas peças, não uma:

- **`hookc`** — binário Rust minúsculo, sem dependência pesada. Lê o payload do
  stdin (ou do argumento, no caso do `notify` do Codex), abre o socket local,
  despeja e sai. Sem parsing além do necessário para rotear, sem I/O de rede, sem
  carregar modelo.
- **`hookd`** — daemon Rust de vida longa. Guarda toda a lógica: normalização,
  política, fila, STT, ponte com o TTS, IPC para CLI e GUI.

Rust pelos três motivos que decidem aqui: binário estático sem runtime para
distribuir nos três sistemas, arranque na casa do milissegundo, e bindings
maduros para o que este projeto precisa em áudio, PTY e ONNX.

## Consequências

**Boas.** O custo do hook fica próximo do irredutível. Daemon morto ou socket
ausente vira uma saída silenciosa com código 0 — o agente não trava. A lógica
mora num lugar só, e CLI, GUI e hooks são clientes iguais.

**Ruins.** Duas peças para instalar, versionar e manter compatíveis; o protocolo
de IPC vira um contrato de verdade, com versão. E há um daemon rodando de fundo,
com tudo que isso implica de ciclo de vida e autostart nos três sistemas.

**Restringe.** Nada que precise de estado entre eventos pode morar no `hookc`.
Toda tentação de "só um cachezinho no cliente" é violação desta decisão.

## Alternativas

**Um binário só, o hook faz tudo.** Descartado: carregar modelo de STT a cada
`PreToolUse` é inviável, e não há como cortar uma fala anterior sem estado.

**Núcleo em Python, junto do `voice-clone`.** Descartado pela latência de
arranque, embora fosse mais rápido de construir. Um cliente fino em shell mais
socket mitigaria, mas ainda deixaria a GUI e o empacotamento no Windows piores.

**Go.** Arranque e distribuição comparáveis; ecossistema de áudio e ASR mais
pobre, e whisper.cpp exigiria CGo de qualquer forma — perdendo a vantagem.

## Revisão — 2026-08-23

A premissa foi medida depois da decisão e se confirmou: o `cvb-hook` em release
custa **1,94 ms por invocação**, incluindo o `fork` do shell e o pipe, com o
daemon de pé. Números e método em [`memory/measurements.md`](../../memory/measurements.md).

A decisão não muda; o registro existe para que ninguém precise remedir, e para
que uma regressão futura tenha uma linha de base.
