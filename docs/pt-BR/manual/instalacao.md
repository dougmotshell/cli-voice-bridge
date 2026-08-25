# Instalação

TODO: escrever de verdade quando houver o que instalar. O esqueleto abaixo é o
fluxo pretendido.

## Antes de começar

1. **`voice-clone` funcionando.** Este projeto não sintetiza nada sozinho.
   Instale-o com o instalador oficial, que não clona repositório nenhum
   (leia o script antes de executar, como com qualquer instalador remoto):

   ```bash
   curl -fsSL https://raw.githubusercontent.com/dougmotshell/voice-clone/main/scripts/install.sh | sh
   ```

   Ele cria o ambiente Python em `~/.local/share/voice-clone` (com o `uv`, que
   baixa o interpretador se faltar), instala ~1,7 GB de dependências e deixa
   os atalhos `voice-clone` e `voice-clone-web` em `~/.local/bin`. Depois
   confira: `voice-clone checar`, e `voice-clone vozes` para ver que existe
   pelo menos uma voz cadastrada — se não houver, `voice-clone cadastrar
   <nome> <audio.wav>` com 6–30 s de fala limpa. A primeira síntese baixa
   1,8 GB de pesos do XTTS-v2, uma única vez; depois nada mais sai da máquina.

   Quem prefere trabalhar a partir de um clone do repositório do `voice-clone`
   também pode: o que importa é uma raiz com `falar.py` e um `.venv/` ao lado.
2. **Pelo menos um dos CLIs instalado** — Claude Code, Codex CLI ou Copilot CLI.
3. **Microfone e saída de áudio** funcionando no sistema.

## Instalar

TODO: preencher. Provável: baixar o binário da release da plataforma, ou
`cargo install --path .` para quem tem Rust. Ver
[portability](../specs/portability.md), seção *Distribuição*.

## Ligar aos CLIs

```bash
cvb install --dry-run            # mostra o que mudaria, sem escrever
cvb install --dry-run --diff     # o mesmo, linha a linha
cvb install                      # aplica nos três
cvb install --cli claude,codex   # ou só nesses
```

Cada arquivo alterado deixa o original em `*.cvb-backup` ao lado. Para desfazer:
`cvb uninstall`, que tira só o que o `cvb` pôs.

A instalação funciona nos dois cenários:

- **Sem hooks anteriores** — o arquivo de configuração não existe ou não tem a
  seção de hooks: o `cvb` cria só o que precisa.
- **Com hooks de terceiros** — `rtk`, por exemplo, mas qualquer outro vale: o
  `cvb` **compõe**, lendo o arquivo, acrescentando a própria entrada e
  preservando o resto. Os hooks que já estavam lá continuam funcionando, e
  nenhum deles é requisito para o `cvb`.

**No Codex há um passo a mais.** Ele guarda um `trusted_hash` do comando de hook
no `config.toml`; ao mudar, ele pede confirmação na próxima sessão. Se o hook
parecer inerte, é isso: abra o Codex e confirme.

## Subir o sidecar de síntese

Sem ele, o projeto fala com a voz do sistema e avisa que foi por aí. Com ele,
fala na sua voz clonada.

```bash
CVB_VOICE_CLONE=~/.local/share/voice-clone \
  ~/.local/share/voice-clone/.venv/bin/python sidecar/servidor.py
```

`~/.local/share/voice-clone` é onde o instalador do `voice-clone` o coloca. Se
você usa um clone do repositório, aponte para ele — o que o sidecar procura é o
`falar.py` na raiz e o interpretador em `.venv/bin/python` (`.venv\Scripts\python.exe`
no Windows). O mesmo caminho pode ficar fixo na configuração, em
`[voice_clone] raiz` — ver [configuração](configuracao.md).

Deixe rodando. A primeira fala leva ~30 s, que é o XTTS-v2 carregando; da segunda
em diante é imediato, e frases repetidas saem do cache.

TODO: supervisão — hoje, se o sidecar morrer, ninguém o levanta de volta.

## Conferir

```bash
cvb doctor
```

Ele checa, e diz em português o que faltou: `voice-clone` no caminho declarado,
voz existente, dispositivo de áudio, hooks instalados nos CLIs ativos, daemon de
pé, e o que a sua plataforma não suporta (por exemplo, atalho global no Wayland
sem portal).

## Desinstalar

```bash
cvb uninstall
```

Remove só o que o `cvb` instalou. Hooks de terceiros ficam onde estavam.
