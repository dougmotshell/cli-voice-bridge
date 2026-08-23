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

Ajuste por momento em vez de calar tudo. `falar = "ausente"` nos momentos de
urgência média costuma resolver: fala só quando você não está olhando. Ver
[configuracao.md](configuracao.md).

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
