# Configuration

> Translation of [`../../pt-BR/manual/configuracao.md`](../../pt-BR/manual/configuracao.md),
> which is the source of truth.

Everything is configurable: which CLIs to apply to, what is spoken, what is
silenced, and whether you answer by voice or by text. The full contract is in
[configuration](../specs/configuration.md); the common tasks are here.

TODO: the commands below do not all exist yet. This is the intended contract.

## Where the file lives

| System | Path |
|---|---|
| Linux | `~/.config/cli-voice-bridge/config.toml` |
| macOS | `~/Library/Application Support/cli-voice-bridge/config.toml` |
| Windows | `%APPDATA%\cli-voice-bridge\config.toml` |

`cvb config edit` opens it in your editor; `cvb config check` validates and points
at the offending line. The GUI edits the same keys and preserves your comments.

The key names are in pt-BR, because they are the contract you write and this
project's prose is pt-BR-first.

## Choose which CLIs it applies to

```toml
[cli.claude]
ativo = true

[cli.copilot]
ativo = false
```

## Choose whether you answer by voice or by text

```toml
[cli.claude]
resposta = "voz"          # "voz" | "texto" | "ambos"
```

Per CLI: you can answer Claude by voice and Codex by text.

## Choose what is spoken and what is not

Configuration speaks in **moments** — the same vocabulary across all three CLIs:

```toml
[momentos."decision.needed"]
falar = "sempre"          # "sempre" | "ausente" | "nunca"

[momentos."tool.started"]
falar = "nunca"
```

`"ausente"` speaks only when you are not looking: window out of focus, or no
keystroke for a few seconds. It is the adjustment that cuts the most noise.

The list of moments and what each one means is in
[event-normalization](../specs/event-normalization.md).

## Profiles

```bash
cvb profile use reuniao    # critical only, and without saying the content
cvb profile use foco       # narration on
```

## Voice

```toml
[geral]
voz = "douglas"      # a voice registered in voice-clone
idioma = "pt-BR"

[voice_clone]
raiz = "~/.local/share/voice-clone"   # where the voice-clone installer puts it
python = ""                           # empty = <raiz>/.venv/bin/python
```

`raiz` is the folder that holds `falar.py`; when empty, the `CVB_VOICE_CLONE`
environment variable applies. `python` only needs a value when the interpreter
is not in the default `.venv/` (on Windows the default is
`<raiz>\.venv\Scripts\python.exe`). Registered voices live in `<raiz>/vozes/`;
`cvb voices` lists the same ones as `voice-clone vozes`.

`cvb say "teste"` confirms the whole chain works.

## Discreet mode

```toml
[privacidade]
modo_discreto = true
```

Speaks the category and never the content: "the agent needs a decision", without
naming the command. Use it when other people can hear.
