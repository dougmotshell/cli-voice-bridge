# O que o voice-clone já aprendeu

`~/www/voice-clone` é dependência externa somente leitura
([ADR-0003](../docs/pt-BR/decisions/0003-tts-delegado-ao-voice-clone.md)). Estas lições
custaram tempo lá e não valem redescobrir aqui.

**Carregar o XTTS-v2 leva cerca de 30 segundos.** É o motivo de o sidecar ser um
processo de vida longa e não um `spawn` por frase. Qualquer proposta que reabra
o modelo por fala está errada por construção.

**Threads = cores físicos, não lógicos.** Com hyperthreading ligado, a mesma
frase levou 2,5× mais tempo. Contraintuitivo e medido. Não "otimize" para
`os.cpu_count()`.

**PyTorch precisa do índice CPU no Linux**, senão arrasta ~2,5 GB de CUDA inútil
numa máquina sem GPU. Se o sidecar algum dia tiver ambiente próprio, herde essa
fixação.

**A licença do XTTS-v2 é CPML: proíbe uso comercial.** Este projeto herda o teto
enquanto depender dele. Aceitável — é pessoal — mas não sugira uso comercial.

**Áudio de voz é dado biométrico.** `vozes/` e `saida/` são ignorados pelo git lá,
e nada de áudio sai da máquina. É o requisito central de lá, e vale igual aqui,
inclusive para gravação de microfone e transcrição.

**Windows quebra por encoding.** Com a saída redirecionada, o encoding padrão é o
do locale e os acentos estouram com `UnicodeEncodeError`. O `falar.py` resolve
com `reconfigure(encoding="utf-8")` no `stdout` e no `stderr`. Herde a lição no
sidecar em vez de redescobrir.

**O diagnóstico oficial de lá é `falar.py checar`.** Antes de suspeitar do
`cli-voice-bridge`, rode aquilo.
