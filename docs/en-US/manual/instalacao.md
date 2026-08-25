# Installation

> Translation of [`../../pt-BR/manual/instalacao.md`](../../pt-BR/manual/instalacao.md),
> which is the source of truth.

TODO: write this properly once there is more to install. The skeleton below is
the intended flow.

## Before you start

1. **A working `voice-clone`.** This project does not synthesize anything on its
   own. Install it with the official installer, which clones no repository
   (read the script before running it, as with any remote installer):

   ```bash
   curl -fsSL https://raw.githubusercontent.com/dougmotshell/voice-clone/main/scripts/install.sh | sh
   ```

   It creates the Python environment in `~/.local/share/voice-clone` (with
   `uv`, which downloads the interpreter if missing), installs ~1.7 GB of
   dependencies, and leaves the `voice-clone` and `voice-clone-web` shortcuts
   in `~/.local/bin`. Then check: `voice-clone checar`, and `voice-clone vozes`
   to see that at least one voice is registered — if there is none,
   `voice-clone cadastrar <name> <audio.wav>` with 6–30 s of clean speech. The
   first synthesis downloads 1.8 GB of XTTS-v2 weights, once; after that
   nothing leaves the machine.

   Working from a clone of the `voice-clone` repository is fine too: what
   matters is a root with `falar.py` and a `.venv/` next to it.
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
cvb install --dry-run --diff     # the same, line by line
cvb install                      # applies to all three
cvb install --cli claude,codex   # or only to those
```

Every file it changes leaves its original as `*.cvb-backup` alongside. To undo:
`cvb uninstall`, which removes only what `cvb` put there.

Installation works in both scenarios:

- **No previous hooks** — the configuration file does not exist or has no hooks
  section: `cvb` creates only what it needs.
- **Third-party hooks present** — `rtk`, for instance, but any other counts:
  `cvb` **composes**, reading the file, appending its own entry, and preserving
  the rest. The hooks that were already there keep working, and none of them is
  a requirement for `cvb`.

**Codex has one extra step.** It stores a `trusted_hash` of the hook command in
`config.toml`; when it changes, it asks for confirmation in the next session. If
the hook seems inert, that is why: open Codex and confirm.

## Start the synthesis sidecar

Without it, the project speaks with the system voice and says so. With it, it
speaks in your cloned voice.

```bash
CVB_VOICE_CLONE=~/.local/share/voice-clone \
  ~/.local/share/voice-clone/.venv/bin/python sidecar/servidor.py
```

`~/.local/share/voice-clone` is where the `voice-clone` installer puts it. If
you use a clone of the repository, point there instead — what the sidecar looks
for is `falar.py` at the root and the interpreter at `.venv/bin/python`
(`.venv\Scripts\python.exe` on Windows). The same path can be fixed in the
configuration, under `[voice_clone] raiz` — see [configuration](configuracao.md).

Leave it running. The first utterance takes ~30 s, which is XTTS-v2 loading; from
the second on it is immediate, and repeated phrases come from the cache.

TODO: supervision — today, if the sidecar dies, nothing brings it back.

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
