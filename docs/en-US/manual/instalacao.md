# Installation

> Translation of [`../../pt-BR/manual/instalacao.md`](../../pt-BR/manual/instalacao.md),
> which is the source of truth.

TODO: write this properly once there is more to install. The skeleton below is
the intended flow.

## Before you start

1. **A working `voice-clone`.** This project does not synthesize anything on its
   own. Check over there first: `.venv/bin/python falar.py checar`, then
   `falar.py vozes` to see that at least one voice is registered.
2. **At least one of the CLIs installed** — Claude Code, Codex CLI, or Copilot CLI.
3. **A working microphone and audio output** on the system.

## Build

```bash
cargo build --release
```

Stable Rust (verified with 1.98.0), 2021 edition. Without Rust on the machine:
`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`

TODO: prebuilt binaries. See [portability](../specs/portability.md), the
*Distribution* section.

## Wire it to the CLIs

```bash
cvb install --dry-run            # shows what would change, without writing
cvb install --cli claude,codex   # installs hooks only for those
```

TODO: `cvb install` is not implemented yet.

Installation **composes** with hooks that already exist: it reads the file,
appends the `cvb` entry, and preserves the rest. If you already use other hooks —
`rtk`, for instance — they keep working.

**Codex has one extra step.** It stores a `trusted_hash` of the hook command in
`config.toml`; when it changes, it asks for confirmation in the next session. If
the hook seems inert, that is why: open Codex and confirm.

## Check

```bash
cvb doctor
```

It checks, and says in Portuguese what is missing: `voice-clone` at the declared
path, an existing voice, the audio device, hooks installed in the active CLIs,
the daemon being up, and what your platform does not support (for example, a
global shortcut on Wayland with no portal).

## Uninstall

```bash
cvb uninstall
```

Removes only what `cvb` installed. Third-party hooks stay where they were.
