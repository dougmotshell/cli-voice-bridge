# Day-to-day use

> Translation of [`../../pt-BR/manual/uso.md`](../../pt-BR/manual/uso.md), which
> is the source of truth.

TODO: write this properly once the project runs end to end. The skeleton below is
the intended behavior.

## Before anything else

```bash
cvb doctor      # says what is missing
cvb voices      # the voices registered in voice-clone
cvb say "teste" # falado — voz clonada | voz do sistema
```

`cvb say` always reports **which path** it spoke through. "voz do sistema" means
the sidecar did not answer — the speech came out, but not in your voice.

## The basics

Open the CLIs the way you already do. With the hooks installed,
`cli-voice-bridge` tells you out loud when the agent needs you: a permission
request, a question, a finished turn, a subagent started or finished.

Nothing changes in how you work — unless you want narration or injected
dictation, which need the wrapper.

## Answering by voice

**Closed question.** When the agent asks for permission, it speaks the question
and opens a listening window. Answer "sim", "não", "sempre", "nunca", "cancelar",
or "opção um/dois/três". The answer returns through the hook, touching no
keyboard.

A destructive command — `rm -rf`, `git push --force`, and whatever is on your
list — is **never** authorized by a single "yes". It asks for one more
confirmation or sends you to the screen. That is deliberate: voice is the
highest-error channel in the system.

**Dictating a prompt.** Hold the global shortcut, speak, release. The transcribed
text goes to the clipboard and you paste it — the default mode, because it works
in any terminal and never breaks.

**Dictating straight into the CLI.** Open the CLI through the wrapper:

```bash
cvb wrap -- claude
```

Then the dictated text is written straight into the CLI's input. Works the same
in the system terminal and in VS Code's or IntelliJ's integrated terminal. By
default it writes and stops, so you can review before sending.

## Muting

```bash
cvb mute 30m      # quiet for half an hour
cvb unmute
```

In the GUI, from the tray icon. Switching to the `reuniao` profile is the finer
equivalent: it silences everything but critical, and critical comes out without
naming the content.

## Seeing what is happening

```bash
cvb events --follow      # the moment stream, live
cvb daemon status
```

The GUI shows the same as a panel, with what is speaking now, what is queued, and
a button to cut the speech.

## When you start typing again

If you submit a prompt while it is speaking, it stops immediately. You are back;
you no longer need the alert.
