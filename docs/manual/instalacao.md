# Instalação

TODO: escrever de verdade quando houver o que instalar. O esqueleto abaixo é o
fluxo pretendido.

## Antes de começar

1. **`voice-clone` funcionando.** Este projeto não sintetiza nada sozinho.
   Confira lá primeiro: `.venv/bin/python falar.py checar`, depois
   `falar.py vozes` para ver que existe pelo menos uma voz cadastrada.
2. **Pelo menos um dos CLIs instalado** — Claude Code, Codex CLI ou Copilot CLI.
3. **Microfone e saída de áudio** funcionando no sistema.

## Instalar

TODO: preencher. Provável: baixar o binário da release da plataforma, ou
`cargo install --path .` para quem tem Rust. Ver
[portability](../specs/portability.md), seção *Distribuição*.

## Ligar aos CLIs

```bash
cvb install --dry-run            # mostra o que seria mudado, sem escrever
cvb install --cli claude,codex   # instala os hooks só nesses
```

A instalação **compõe** com os hooks que já existem: ela lê o arquivo,
acrescenta a entrada do `cvb` e preserva o resto. Se você já usa outros hooks —
`rtk`, por exemplo — eles continuam funcionando.

**No Codex há um passo a mais.** Ele guarda um `trusted_hash` do comando de hook
no `config.toml`; ao mudar, ele pede confirmação na próxima sessão. Se o hook
parecer inerte, é isso: abra o Codex e confirme.

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
