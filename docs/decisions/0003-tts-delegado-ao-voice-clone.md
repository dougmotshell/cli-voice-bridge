# ADR-0003 — Síntese de voz delegada ao voice-clone, por sidecar

**Status:** aceito — 2026-08-23
**Nível C4:** [contêiner](../architecture/02-container.md)
**Specs que esta decisão move:** [speech-output](../specs/speech-output.md)

## Contexto

A voz clonada já existe: `~/www/voice-clone` roda XTTS-v2
100% offline em CPU, em pt-BR e en-US, e é a razão de este projeto existir.
XTTS-v2 é Python, e carregar o modelo leva cerca de 30 segundos.

Reimplementar em Rust não está na mesa: as bibliotecas do XTTS são Python e a
qualidade da voz é o produto.

## Decisão

O `voice-clone` é **dependência externa somente leitura**. Um sidecar Python de
vida longa carrega o modelo uma vez e atende o `hookd` por socket local. A
integração se dá pelo contrato público do `voice-clone`, com o caminho vindo da
configuração — nunca embutido no código, nunca por cópia de arquivo.

Há sempre um caminho de degradação: sidecar indisponível cai para a voz do
sistema operacional, com aviso na GUI e no `cvb doctor`. Falar feio é melhor que
não avisar que o agente está travado esperando permissão.

## Consequências

**Boas.** Zero duplicação da parte difícil. O `voice-clone` evolui sozinho.
A restrição de privacidade dele — nada de áudio sai da máquina — é herdada.

**Ruins.** Um runtime Python a mais para instalar e manter vivo. O sidecar
precisa de supervisão: morreu, reinicia. E o projeto herda a licença CPML do
XTTS-v2, que proíbe uso comercial — aceitável, porque este projeto é pessoal e
sem fim comercial, mas é um teto real.

**Restringe.** Nada aqui pode fazer `import vozclone` como se fosse módulo
próprio nem editar o `voice-clone`. Necessidade de mudar lá é uma conversa
separada, com ADR lá.

## Alternativas

**Um `spawn` de `falar.py` por frase.** Descartado: pagaria os ~30 s de carga do
modelo em cada fala.

**TTS de nuvem (ElevenLabs, OpenAI).** Descartado: manda o texto do trabalho para
fora, custa dinheiro e joga fora a voz clonada.

**Só a voz do sistema.** Descartado como padrão — é o fallback.

**Absorver o `voice-clone` para dentro deste repositório.** Descartado: duplica
manutenção e mata a evolução independente dos dois.
