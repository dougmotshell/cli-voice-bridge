# Configuração

Tudo é configurável: em quais CLIs aplicar, o que falar, o que calar, e se você
responde por voz ou por texto. O contrato completo está em
[configuration](../specs/configuration.md); aqui ficam as tarefas comuns.

TODO: os comandos abaixo ainda não existem. É o contrato pretendido.

## Onde fica o arquivo

| Sistema | Caminho |
|---|---|
| Linux | `~/.config/cli-voice-bridge/config.toml` |
| macOS | `~/Library/Application Support/cli-voice-bridge/config.toml` |
| Windows | `%APPDATA%\cli-voice-bridge\config.toml` |

`cvb config edit` abre no seu editor; `cvb config check` valida e aponta a linha
do erro. A GUI edita as mesmas chaves e preserva os seus comentários.

## Escolher em quais CLIs aplicar

```toml
[cli.claude]
ativo = true

[cli.copilot]
ativo = false
```

## Escolher se responde por voz ou por texto

```toml
[cli.claude]
resposta = "voz"      # "voz" | "texto" | "ambos"
```

Vale por CLI: você pode responder ao Claude por voz e ao Codex por texto.

## Escolher o que fala e o que cala

A configuração fala em **momentos** — o mesmo vocabulário nos três CLIs:

```toml
[momentos."decision.needed"]
falar = "sempre"      # "sempre" | "ausente" | "nunca"

[momentos."tool.started"]
falar = "nunca"
```

`"ausente"` fala só quando você não está olhando: janela fora de foco ou sem
tecla há alguns segundos. É o ajuste que mais reduz ruído.

A lista dos momentos e o que cada um significa está em
[event-normalization](../specs/event-normalization.md).

## Perfis

```bash
cvb profile use reuniao    # só o crítico, e sem dizer o conteúdo
cvb profile use foco       # narração ligada
```

## Voz

```toml
[geral]
voz = "douglas"      # nome cadastrado no voice-clone
idioma = "pt-BR"
```

`cvb say "teste"` confirma que a cadeia inteira funciona.

## Modo discreto

```toml
[privacidade]
modo_discreto = true
```

Fala a categoria e nunca o conteúdo: "o agente precisa de uma decisão", sem dizer
qual comando. Use quando houver outras pessoas ouvindo.
