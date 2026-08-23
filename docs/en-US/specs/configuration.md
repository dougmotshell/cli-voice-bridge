# Spec — Configuration

> Translation of [`../../pt-BR/specs/configuration.md`](../../pt-BR/specs/configuration.md),
> which is the source of truth.

**Capability:** make everything configurable — per CLI, per project, per moment —
without turning the configuration file into a puzzle.

**ADRs constraining this spec:** [ADR-0007](../decisions/0007-esquema-canonico-de-momentos.md).
**C4 level:** [component](../architecture/03-component.md) — `core::config`.

## Problem

The request is explicit: choose which CLIs to apply to, whether to answer by
voice or by text, what is spoken and what is not. That is a large space of
combinations. Without a clear precedence structure it becomes a file nobody
understands and a GUI that does not match the CLI.

## Scope

In: format, location, precedence, the set of keys, validation, and editing from
both interfaces.
Out: the meaning of each policy — that is in the [output](speech-output.md) and
[input](speech-input.md) specs.

## Layers, weakest to strongest

1. Defaults compiled into the binary.
2. The person's configuration: `~/.config/cli-voice-bridge/config.toml` (Linux),
   `~/Library/Application Support/cli-voice-bridge/` (macOS),
   `%APPDATA%\cli-voice-bridge\` (Windows).
3. The project's configuration: `.cli-voice-bridge.toml` at the repository root.
4. The active profile (`cvb profile use <name>`) — e.g. `reuniao` silences
   everything but critical; `foco` turns narration on.
5. `CVB_*` environment variables.
6. Command-line arguments.

A project's configuration **cannot** enable what the person disabled for safety
(microphone, voice authorization of a destructive command). A cloned repository
does not control your microphone.

## Format

TOML. The same format as Codex's `config.toml`, readable and diffable.

```toml
[geral]
voz = "douglas"                 # a voice registered in voice-clone
idioma = "pt-BR"
perfil = "padrao"
segundos_de_relevancia = 30     # older than this is not spoken; critical never expires

[voice_clone]
raiz = "~/www/voice-clone"      # never embedded in code
python = "{raiz}/.venv/bin/python"

[cli.claude]
ativo = true
transportes = ["hook", "pty"]
resposta = "voz"          # "voz" | "texto" | "ambos"

[cli.codex]
ativo = true
transportes = ["hook", "notify"]
resposta = "texto"

[cli.copilot]
ativo = false

[momentos."decision.needed"]
falar = "sempre"          # "sempre" | "ausente" | "nunca"
interrompe = true
molde = "{cli} wants to run {ferramenta}. Allow?"

[momentos."tool.started"]
falar = "nunca"

[escuta]
acionamento = "push-to-talk"
atalho = "Ctrl+Alt+Space"
motor = "TODO"
confirmar_destrutivo = true

[privacidade]
modo_discreto = false
redigir = ["token", "senha", "chave", "Bearer "]
retencao_log_dias = 7
```

Key names stay in pt-BR because they are the contract the person writes, and the
project's prose is pt-BR-first. TODO: freeze the schema once the core exists. The
table above is the intent, not a closed contract — the only part that is already
a contract is the precedence.

## Validation

`cvb config check` validates and explains what is wrong in pt-BR, pointing at the
line. An unknown key is a **warning**, not fatal: configuration from a newer
version must not break an older one. An invalid value is an error.

`cvb doctor` goes further: it checks that `voice-clone` is at the declared path,
that the voice exists, that the audio device responds, that hooks are installed
in the CLIs marked active, and that the daemon is up.

## CLI ↔ GUI parity

Every key is editable from both interfaces. The GUI writes the same TOML, with
comments preserved — someone who edited by hand does not lose what they wrote.
Hot reload: a file change reloads the daemon without dropping the queue.

## Alternatives considered

**JSON.** Rejected: no comments, and the file is meant to be read and edited by
hand. **YAML.** Rejected: type ambiguities that produce configuration bugs.
**A database edited only through the GUI.** Rejected: it kills versioning and
diffing, and forces opening a GUI for a one-line change.

## Test plan

TODO: write it. Minimum: a precedence test layer by layer; a test that a
project's configuration cannot enable the microphone or disable destructive
confirmation; and a GUI → file round-trip test preserving comments.

## Open questions

- The final schema (above).
- Where profiles live: sections in the same file or separate files.
