> Translation of [`SKILL.md`](SKILL.md), the source of truth. The generator
> ignores this file; only `SKILL.md` is projected into the CLI surfaces.

# Voice smoke test

No automated test judges whether a voice sounds good. An audio change is not done
until it has been heard.

## Output

1. `cvb doctor` — it has to pass clean. If it flags something, stop and fix that
   first.
2. `cvb say "teste de voz do cli-voice-bridge"` — listen. Is it the right voice?
   Does it sound natural? How long between the command and the first sound?
3. **Fire a real moment**, not a simulated one: open an AI CLI and provoke a
   permission request. `cvb events --follow` in a window alongside shows the
   moment arriving.
4. **Cutting:** while it is speaking something long, provoke a critical moment.
   The previous utterance must be cut, not queued.
5. **Fallback:** kill the sidecar and repeat step 2. It must speak with the
   system voice and warn — never go mute.

## Input

6. `cvb listen` — say a technical sentence in pt-BR and check the transcription.
7. **Closed question:** provoke a permission request and answer "sim" by voice.
   Confirm the decision reached the CLI.
8. **Destructive refusal:** provoke a request for a destructive command and answer
   "sim". It must **not** authorize on a single yes.
9. **Dictation:** hold the shortcut, dictate a sentence, check the configured
   destination (clipboard or the CLI's input).

## What to report

Say what you heard and measured, not what you expected to hear. Latency to first
sound and naturalness are the two things only an ear judges. If some step could
not be run, say which and why — a half-done smoke test reported as complete is
worse than none.
