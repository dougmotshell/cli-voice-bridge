# ADR-0009 — Reproduzir áudio pelo reprodutor do sistema, não por biblioteca

**Status:** aceito — 2026-08-23
**Nível C4:** [componente](../architecture/03-component.md)
**Specs que esta decisão move:** [speech-output](../specs/speech-output.md), [portability](../specs/portability.md)

## Contexto

O sidecar entrega um arquivo WAV; alguém precisa tocá-lo. A resposta idiomática
em Rust seria `rodio`, que traz `cpal` e um decodificador junto.

Só que `cpal` no Linux compila contra o ALSA: sem `libasound2-dev` instalado, o
build falha — e falha num projeto que a pessoa acabou de clonar, antes de
qualquer coisa funcionar. Numa máquina de desenvolvimento isso é um `apt install`
de distância; para quem só quer rodar, é uma parede logo na porta.

Este projeto ainda vai precisar de captura de áudio para a entrada por voz, e aí
uma biblioteca nativa provavelmente será inevitável. Mas reprodução de um WAV é o
caso mais simples possível, e todos os três sistemas já vêm com um programa que
faz exatamente isso.

## Decisão

Reproduzir invocando um programa do sistema, escolhido na primeira execução entre
uma lista de candidatos por plataforma:

| Sistema | Candidatos, em ordem |
|---|---|
| Linux | `paplay`, `pw-play`, `aplay`, `ffplay` |
| macOS | `afplay` |
| Windows | `powershell` com `System.Media.SoundPlayer` |

A pessoa pode fixar o comando em `geral.reprodutor` na configuração, e aí a lista
não é consultada. Nenhum reprodutor encontrado é uma falha **relatada** pelo
`cvb doctor`, com a lista do que foi procurado — não um erro genérico.

## Consequências

**Boas.** Zero dependência de compilação: `cargo build` funciona numa máquina
recém-clonada, sem pacote de sistema. Um problema de áudio fica reproduzível na
mão — a pessoa roda o mesmo comando no terminal e vê o que acontece. E o
reprodutor do sistema já respeita as configurações de saída do usuário.

**Ruins.** Um processo por fala, com o custo de arranque que isso tem — pequeno
perto dos ~30 s do modelo, mas não é zero. Não dá para controlar volume, fazer
fade nem misturar áudios. E **cortar a fala em curso** vira matar um processo
filho, que é mais grosseiro que parar um fluxo de áudio.

**Restringe.** Enquanto valer esta decisão, a fila de voz corta interrompendo um
processo, não pausando um fluxo. Se o corte ficar audivelmente ruim, é sinal de
que chegou a hora de revisitar — e aí provavelmente junto com a captura de áudio,
numa decisão só.

## Alternativas

**`rodio`/`cpal`.** Descartada por ora: exige `libasound2-dev` no Linux e
transformaria o primeiro `cargo build` numa caça a pacote de sistema. Volta à
mesa quando a captura de áudio entrar, porque aí a dependência nativa já terá
sido paga.

**Tocar dentro do sidecar Python.** Descartada: o sidecar existe para carregar o
XTTS uma vez, e dar a ele o papel de saída de áudio confundiria as duas coisas.
Além disso o fallback para a voz do sistema precisa tocar **sem** o sidecar,
justamente porque ele pode estar morto.

**Um servidor de áudio próprio.** Descartada: complexidade sem retorno para tocar
um WAV por vez.
