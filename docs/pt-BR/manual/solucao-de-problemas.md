# Solução de problemas

**Primeiro passo, sempre:** `cvb doctor`. Ele confere a cadeia inteira e diz em
português o que quebrou. Só depois investigue o resto.

TODO: preencher com os problemas reais assim que houver uso. O que segue é o que
já se sabe que vai acontecer, pela natureza das peças.

## Não fala nada

1. `cvb daemon status` — o daemon está de pé? `hookc` sem daemon sai em silêncio
   de propósito, para nunca travar o agente.
2. `cvb doctor` — os hooks estão mesmo instalados nos CLIs marcados como ativos?
3. **No Codex:** ele guarda um `trusted_hash` do comando de hook no
   `config.toml`. Se o comando mudou, o hook fica inerte até você confirmar numa
   sessão do Codex.
4. Você está com o perfil `reuniao` ou com `cvb mute` ligado?
5. O momento em questão está com `falar = "nunca"` ou `"ausente"` na sua
   configuração?

## Fala com a voz errada, ou robótica

É o fallback: o sidecar do XTTS não subiu e ele caiu para a voz do sistema, de
propósito — avisar com voz feia é melhor que não avisar. `cvb doctor` diz por
quê. Causas prováveis: caminho do `voice-clone` errado na configuração, venv de
lá quebrado (`falar.py checar` no `voice-clone` resolve isso), ou o nome da voz
não existe (`falar.py vozes`).

## A primeira fala demora muito

O XTTS-v2 leva cerca de 30 segundos para carregar. O sidecar carrega uma vez e
fica vivo; se ele está reiniciando a cada fala, é defeito — veja o log.

## Fala demais

Ajuste por momento em vez de calar tudo. Ver [configuracao.md](configuracao.md).

**Atenção: `falar = "ausente"` ainda não funciona como deveria.** Ele deveria
falar só quando você não está olhando, mas a detecção de presença não existe —
então hoje `"ausente"` simplesmente **cala**. É o fallback documentado (assumir
presente e falar menos), não um defeito silencioso. Enquanto isso, use
`"sempre"` no que você quer ouvir e `"nunca"` no resto. O que falta está em
[presence-detection](../specs/presence-detection.md).

## O cache de áudio só cresce

É verdade, e é lacuna conhecida: ele guarda um WAV por frase e nunca apaga nada.
Na prática cresce devagar, porque as frases se repetem — mas mensagem de
assistente resumida é única a cada vez, e trocar de voz invalida tudo sem apagar.

Para limpar à mão:

| Sistema | Diretório |
|---|---|
| Linux | `~/.local/share/cli-voice-bridge/cache-audio/` |
| macOS | `~/Library/Application Support/cli-voice-bridge/cache-audio/` |
| Windows | `%LOCALAPPDATA%\cli-voice-bridge\cache-audio\` |

Apagar o diretório é seguro: a próxima fala sintetiza de novo. O teto automático
está por decidir — ver [speech-output](../specs/speech-output.md).

## O daemon morreu e deixou coisa para trás

Morte por sinal não roda a limpeza, então o socket fica no disco. Não é grave: o
arranque seguinte detecta que ninguém atende naquele endereço e remove sozinho.
Se um áudio ficou tocando depois de o daemon morrer, não há como cortá-lo pelo
`cvb` — espere terminar ou mate o processo do reprodutor.

Encerramento ordenado é lacuna conhecida; o que falta está em
[daemon-lifecycle](../specs/daemon-lifecycle.md).

## Não escuta

1. `cvb listen` — mostra a transcrição do que ele ouviu. Se não ouve nada, é
   dispositivo; se ouve errado, é modelo.
2. Dispositivo de entrada certo na configuração? `cvb doctor` confere.
3. **Atalho global não funciona:** no Wayland ele depende do portal do sistema;
   onde não existe, não funciona, e o `doctor` diz isso. Use o ditado pela área
   de transferência. No macOS, o atalho e a injeção de teclado exigem permissão
   de Acessibilidade, concedida por você nas Preferências.

## Entende errado o "sim" e o "não"

O modo de pergunta fechada usa vocabulário restrito justamente para isso. Se
ainda erra: fale mais perto, reduza ruído, e confira o motor de STT configurado.
Abaixo do limiar de confiança ele não decide — pergunta de novo ou devolve para
a tela. Isso é o comportamento correto, não um defeito.

## O CLI ficou estranho depois do `cvb wrap`

O wrapper PTY é o transporte frágil por natureza: ele lê a tela, e qualquer
redesenho de TUI muda o que ele vê. `cvb doctor --pty` testa as regras contra a
versão instalada do CLI. Enquanto não houver correção, desligue o `pty` na
configuração daquele CLI e abra o CLI direto — os hooks continuam funcionando
sozinhos.

## O agente ficou lento

Não deveria: o `hookc` sai em milissegundos e nunca espera pelo daemon. Se ficou
lento depois de instalar, é defeito sério — abra uma issue com o log e desligue
os hooks (`cvb uninstall`) enquanto isso.
