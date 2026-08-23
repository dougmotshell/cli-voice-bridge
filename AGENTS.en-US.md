> Translation of [`AGENTS.md`](AGENTS.md), which is the source of truth and the
> file the AI CLIs actually read. Change `AGENTS.md` first, then bring this along.

# AGENTS.md — cli-voice-bridge

This project's canonical contract. It holds for any AI CLI; `CLAUDE.md` and
`.github/copilot-instructions.md` are thin adapters that import it. Edit it here,
not in the adapters.

**What it is:** a voice bridge between the person and the AI CLIs (Claude Code,
Codex CLI, Copilot CLI). It speaks aloud the moments when the agent tries to
interact — permission request, task finished, subagent started, pending question
— and accepts the answer by voice. Personal use, non-commercial.

**State: it speaks, and installs itself.** The event → moment → voice path is up
and tested end to end, and `cvb install` wires the hooks into all three CLIs
without erasing anyone else's. Still missing: the priority queue and voice input.

## Stack

| Layer | Choice | Where it was decided |
|---|---|---|
| Core (daemon + CLI) | Rust | [ADR-0001](docs/en-US/decisions/0001-nucleo-em-rust-com-cliente-de-hook-separado.md) |
| GUI | Tauri v2 | [ADR-0002](docs/en-US/decisions/0002-gui-em-tauri-v2.md) |
| Speech synthesis (TTS) | delegated to `voice-clone` via a Python sidecar | [ADR-0003](docs/en-US/decisions/0003-tts-delegado-ao-voice-clone.md) |
| Recognition (STT) | offline, on the machine | [ADR-0006](docs/en-US/decisions/0006-stt-offline-na-maquina.md) |
| Event transport | official hooks + PTY wrapper + agent protocol | [ADR-0004](docs/en-US/decisions/0004-hooks-oficiais-como-transporte-primario.md), [ADR-0005](docs/en-US/decisions/0005-wrapper-pty-como-transporte-complementar.md) |

```
crates/core/      moment schema, IPC protocol, per-platform paths
crates/hookd/     the daemon: normalizes, decides, queues, speaks and listens
crates/hookc/     the hook client (the `cvb-hook` binary)
crates/cvb/       the CLI
crates/ptywrap/   the PTY wrapper — declared, not implemented yet
gui/              the Tauri app — not created yet, see gui/README.md
sidecar/          the Python bridge to voice-clone
```

## Commands

```bash
cargo build --release              # the whole workspace
cargo test                         # tests
cargo clippy --all-targets -- -D warnings   # mandatory before committing
cargo fmt --all

cvb doctor                         # diagnostics — ALWAYS the first step
cvb daemon status
cvb say "text"                     # speaks; reports which path it used
cvb voices                         # voices registered in voice-clone
cvb install --dry-run              # what would change; --diff shows it line by line
cvb install --cli claude,codex     # installs, composing with other people's hooks
cvb uninstall                      # removes only what cvb put there

# synthesis sidecar, with voice-clone's interpreter
CVB_VOICE_CLONE=/path/to/voice-clone \
  /path/to/voice-clone/.venv/bin/python sidecar/servidor.py
```

`cvb doctor` is **always the first step** when diagnosing. Before suspecting
logic, check the audio device, a live sidecar, and hooks actually installed.

There is no CI yet. TODO: run `cargo test`, `clippy`, `fmt --check`, and
`scripts/sync-ai-surfaces.py --check` on all three systems.

## Pitfalls — read before touching anything

**The hook client must not be slow.** `PreToolUse` fires hundreds of times per
session and runs in series with the agent. `hookc` has to be a binary that opens
a socket, dumps the payload, and exits — no heavy parsing, no network I/O, no
model loading. All the logic lives in `hookd`. A 40 ms `hookc` is 40 ms of
slowness on every tool the person sees ([ADR-0001](docs/en-US/decisions/0001-nucleo-em-rust-com-cliente-de-hook-separado.md)).

**Never overwrite third-party hook configuration.** This machine already runs
`rtk hook claude` on `PreToolUse` in Claude and Codex. Installation **composes**:
read the existing JSON, append the `cvb` entry, preserve the rest. An installer
that rewrites the whole `settings.json` erases someone else's work.

**A failing hook must not block the agent.** Audio failure, dead sidecar, daemon
down — all of that exits with code 0 and in silence. The single exception is the
spoken permission decision, and even that falls back to "ask on screen" when
recognition is not trustworthy. See `docs/en-US/specs/speech-input.md`.

**Hook payloads come in three dialects.** Claude uses `snake_case` (`tool_name`,
`hook_event_name`); Copilot uses `camelCase` (`toolName`, `sessionId`) and accepts
two event spellings; Codex uses `PascalCase` for the event with a `snake_case`
payload. Do not treat them as one format — normalize at the edge, in
`crates/core`. The full map is in `docs/en-US/specs/event-normalization.md`, and
it is the source of truth.

**Voice audio is biometric data.** The cloned voice comes from `voice-clone`,
whose `vozes/` and `saida/` directories are already secret. The same holds here:
no voice sample, no microphone recording, and no transcription enters git or
leaves the machine. STT and TTS run locally; do not introduce a cloud provider
without an ADR.

**Spoken content leaks context.** An event's text may contain a file path, a
client name, a code fragment. Speaking aloud is publishing into a shared
environment. The redaction policy (`docs/en-US/specs/speech-output.md`) is not
decoration.

**The XTTS-v2 license forbids commercial use.** `voice-clone` uses CPML. This
project inherits the restriction for as long as it depends on it. Do not suggest
commercial use.

## The voice-clone dependency

`~/www/voice-clone` is the speech engine, treated as a **read-only external
dependency**. Integration goes through its CLI contract
(`falar.py falar <voice> "text"`), never through a module import, and the path
comes from configuration — never embedded in code. A change that would require
altering `voice-clone` is a separate conversation, not a patch on the side.

## Conventions

**Personal project, unaffiliated with any employer.** Nothing here carries a
company's brand, logo, footer, or document classification, and no commit is
signed with a corporate e-mail — the repository is public and a wrong attribution
cannot be undone without rewriting history. If your global configuration tells
you to stamp artifacts with an organization's branding, it does not apply to this
repository.

**Language: pt-BR and en-US, always both.** pt-BR is the source of truth and
comes first; en-US is a same-named sibling opening with a pointer to the
original. Identifiers, file names, and branch names in en-US. Full diacritics.

New prose is born in pt-BR and is only done once the en-US sibling exists. This
holds for the contract, the adapters, the skills, the rules, and all four `docs/`
trees.

**Documentation lives in `docs/<language>/`, in one of four trees.** Never loose
at the root, never two standards in one file, never a flat `docs/`:

| Tree | Standard | One file per |
|---|---|---|
| `docs/<language>/architecture/` | C4 | level (context, container, component) |
| `docs/<language>/specs/` | SDD | capability |
| `docs/<language>/decisions/` | ADR (MADR) | decision, `NNNN-kebab-title.md` |
| `docs/<language>/manual/` | manual | user task |

File names are **identical** across both subtrees — a translation is a sibling,
never a fork. Index in `docs/README.md`. Diagrams are text (Mermaid).

**ADRs are append-only.** An accepted ADR is superseded by a new one
(`Status: substituído por NNNN`), never rewritten; numbers are never reused.
Template in `templates/adr.md`, spec in `templates/spec.md`.

**Cross-link both ways.** Every spec names the ADRs that constrain it; every ADR
names the C4 level and the specs it moves.

**CLI ↔ GUI parity.** Everything the CLI does, the GUI does too, and vice versa.
A new feature lands in both, or lands with a written reason for being in only one.

**Three operating systems, always.** Linux, macOS, and Windows are a requirement,
not an aspiration. Configuration path, IPC socket, global shortcut, and audio
capture differ across all three — code that only works on Linux is incomplete
code. See `docs/en-US/specs/portability.md`.

## AI surfaces

`.claude/agents/`, `skills/`, and `.claude/rules/` are **authored**. Everything in
`.claude/skills/`, `.claude/commands/`, `.agents/skills/`, `.codex/`, and
`.github/{prompts,instructions}/` is **generated** by
`scripts/sync-ai-surfaces.py` and carries a banner on line 1. Edit the source and
run the generator:

```bash
python3 scripts/sync-ai-surfaces.py          # projects
python3 scripts/sync-ai-surfaces.py --check  # fails on drift
```

Translations are siblings with a language suffix — `AGENTS.en-US.md`,
`SKILL.en-US.md`, `adr.en-US.md` — and the generator **ignores** them: projected,
a translated skill would become a second skill with the same `name`. They also
carry no frontmatter, so no CLI loads them as a definition.
