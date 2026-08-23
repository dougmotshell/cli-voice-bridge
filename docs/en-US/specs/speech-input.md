# Spec — Speech input

> Translation of [`../../pt-BR/specs/speech-input.md`](../../pt-BR/specs/speech-input.md),
> which is the source of truth.

**Capability:** answer the agent by speaking, in every way that makes sense —
from "yes" to a full dictated prompt.

**ADRs constraining this spec:** [ADR-0005](../decisions/0005-wrapper-pty-como-transporte-complementar.md),
[ADR-0006](../decisions/0006-stt-offline-na-maquina.md).
**C4 level:** [component](../architecture/03-component.md) — `hookd::listen`.

## Problem

Speaking is easy; delivering what was said to the right process is not. The CLI
is in an interactive TUI, possibly in an IDE's integrated terminal, with no input
API. And misrecognizing a "yes" can authorize an `rm -rf`.

## Scope

In: microphone capture, speech detection, offline transcription, the four answer
modes, and the confirmation policy.
Out: synthesis ([speech-output](speech-output.md)) and event transport
([capture-transports](capture-transports.md)).

## The four modes

All implemented; the configuration chooses which are active, per CLI.

### 1. Closed answer — the safest

When the moment is `decision.needed`, the daemon speaks the question and opens a
listening window with a **restricted vocabulary**: yes, no, always, never, option
one/two/three, cancel. A closed vocabulary is nearly infallible, and the decision
returns through the hook itself — Claude's and Copilot's `PermissionRequest`
accept `decision`/`behavior` in the response, and `PreToolUse` accepts
`permissionDecision`. It goes through no keyboard at all.

**Safety rule.** A command classified as destructive (`rm -rf`,
`git push --force`, `DROP`, anything the configuration marks) is **never**
authorized by voice alone: the daemon asks for a code word to be repeated, or
refuses and sends the decision to the screen. Voice is the highest-error channel
in the system; dangerous authorization does not travel it unaccompanied.

Outside the listening window, nothing is heard. The microphone does not stay open.

### 2. Dictation to the clipboard

A global shortcut records, transcribes, puts the text on the clipboard, and
notifies. The person pastes and checks before sending. Works with any CLI, in any
terminal, with no wrapper and no special permission. It is the default mode
because it is the one that never breaks.

### 3. Dictation injected into the CLI

With the PTY wrapper active (`cvb wrap -- claude`), the transcribed text is
written straight into the CLI's `stdin`. No simulated keystrokes, no dependence
on window focus, the same in the system terminal and in VS Code's or IntelliJ's.

Two configurable variants: **write and stop** (the person reviews and presses
Enter) or **write and send**. The second is the most fluid and the riskiest; the
default is the first.

Without the wrapper there is OS-level keystroke simulation — it needs
Accessibility permission on macOS and is fragile on Wayland. It stays available,
marked as the lowest-confidence path.

### 4. Conversation over a protocol

In `cvb console --cli <name>` mode the project is the ACP/app-server client:
speech becomes a protocol message, with no keyboard and no PTY. It is the
cleanest path, and it costs the original TUI. See
[capture-transports](capture-transports.md).

## Recognition chain

```
microphone → capture → VAD → speech segmentation → STT → post-processing → destination
```

- **Capture.** Rate and channels fixed at the model's input; the device is chosen
  in configuration and checked by `cvb doctor`.
- **VAD.** Detects speech start and end so silence is not transcribed and the
  person is not cut off mid-sentence. Silero VAD is the working hypothesis.
- **STT.** Offline, on the machine, pt-BR and en-US
  ([ADR-0006](../decisions/0006-stt-offline-na-maquina.md)). TODO: choose between
  `whisper.cpp` (truly multilingual, high quality, heavier) and `sherpa-onnx`
  (lighter, real streaming). Measure on the target machine before deciding —
  `voice-clone` already showed that measuring changes the answer.
- **Post-processing.** In closed mode, match against the vocabulary and return a
  confidence; below the threshold, do not decide — ask again or hand it back to
  the screen. In dictation, apply a dictionary of technical terms the model gets
  wrong (tool names, project acronyms), configurable.

## Triggering

- **Push-to-talk** with a global shortcut: records while the key is held. This is
  the default — no open microphone, no accidental listening.
- **Automatic listening window** after a critical moment, time-limited.
- **Wake word:** TODO: decide whether it is in scope. It requires an
  always-open microphone, which contradicts the minimal-listening principle.
  Probably not.

## Privacy

- The microphone opens only on an explicit action or inside an announced
  listening window. The GUI and the CLI show, unmistakably, when it is recording.
- Microphone audio is **discarded** right after transcription. There is no
  persistent recording unless the person explicitly enables it, and even then it
  stays out of git.
- Transcription does not leave the machine.
- Voice is biometric data: the same regime as `voice-clone` applies here.

## Alternatives considered

**Cloud STT.** Rejected: it sends the voice and the work's content outside,
against the project's central principle.

**Free dictation only, no closed mode.** Rejected: the most frequent answer is
"yes" or "no" to a permission question, and that is exactly where an error is
expensive.

**Closed mode only.** Rejected: the person asked for every form, and dictating a
long prompt is half the value.

## Test plan

TODO: write it. Minimum: a battery of recorded "yes"/"no"/"cancel" clips in
pt-BR with noise, asserting classification and confidence; a test that a
destructive command is never authorized by a single "yes"; and a PTY injection
test with an echo program standing in for the CLI.

## Open questions

- The STT engine (above).
- The wake word (above).
- How to signal "I am recording" when no GUI is open and the terminal is in
  another window. Likely: a short start and stop sound, plus the GUI indicator.
