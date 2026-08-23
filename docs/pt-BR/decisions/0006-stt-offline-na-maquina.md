# ADR-0006 — Reconhecimento de fala offline, na máquina

**Status:** aceito quanto ao princípio; **motor em aberto** — 2026-08-23
**Nível C4:** [componente](../architecture/03-component.md)
**Specs que esta decisão move:** [speech-input](../specs/speech-input.md)

## Contexto

Responder por voz exige transcrever. O que se dita para um agente de código é o
trabalho em si: nome de repositório, caminho, trecho de código, às vezes nome de
cliente. Mandar isso para um serviço de transcrição é mandar o trabalho para
fora.

O `voice-clone` já estabeleceu o princípio para o lado da síntese: nada de áudio
sai da máquina, e isso é o requisito central, não uma preferência. Não faria
sentido honrar isso na saída e violar na entrada.

## Decisão

STT roda **na máquina, offline**, com suporte a pt-BR e en-US. Nenhum áudio de
microfone e nenhuma transcrição sai da máquina. O áudio é descartado logo após a
transcrição.

**O motor fica em aberto.** Os candidatos e o que os separa:

| Motor | A favor | Contra |
|---|---|---|
| `whisper.cpp` | multilíngue de verdade, qualidade alta, binding Rust maduro, roda bem em CPU | modelo maior, latência maior, streaming é aproximação |
| `sherpa-onnx` | mais leve, streaming real, várias famílias de modelo | qualidade em pt-BR varia muito conforme o modelo |
| Vosk | leve, maduro, offline por design | qualidade abaixo do Whisper em pt-BR |
| Moonshine | latência muito baixa em CPU | só inglês — elimina para o uso principal |

A decisão sai de **medição na máquina alvo**, não de tabela comparativa. Foi
exatamente assim que o `voice-clone` escolheu o XTTS-v2, e foi medindo que ele
descobriu que threads = cores físicos, não lógicos.

## Consequências

**Boas.** O trabalho não vaza. Funciona sem rede. Sem custo por minuto.

**Ruins.** Pesos de modelo para baixar e versionar fora do git. Qualidade abaixo
de um serviço de nuvem, especialmente em pt-BR com termo técnico. Custo de CPU
concorrendo com o XTTS, que também é CPU.

**Restringe.** Não se introduz provedor de nuvem de STT sem um ADR que substitua
este. É por causa desta restrição que o modo de resposta fechada
([speech-input](../specs/speech-input.md)) usa vocabulário restrito: com um
modelo local menor, vocabulário fechado é o que torna a decisão confiável.

## Alternativas

**STT de nuvem.** Descartado pelo motivo acima.

**Sem entrada por voz, só saída.** Descartado: a pessoa pediu explicitamente
todas as formas de resposta.

**Reconhecimento nativo do sistema** (Ditado do macOS, Reconhecimento de Fala do
Windows). Descartado: qualidade irregular, disponibilidade irregular, e no macOS
o ditado avançado pode envolver servidor da Apple.

## Revisão

TODO: fechar o motor. Sai de um teste com áudio real em pt-BR, medindo latência e
taxa de erro nas duas situações que importam: vocabulário fechado ("sim", "não",
"cancelar") e ditado de um prompt técnico. Quando decidido, escrever ADR-00NN que
substitui este.
