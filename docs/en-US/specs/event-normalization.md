# Spec — Event normalization

> Translation of [`../../pt-BR/specs/event-normalization.md`](../../pt-BR/specs/event-normalization.md),
> which is the source of truth.

**Capability:** turn the events of three AI CLIs, with three different dialects,
into a single stream of "moments" that the speech layer understands.

**ADRs constraining this spec:** [ADR-0004](../decisions/0004-hooks-oficiais-como-transporte-primario.md),
[ADR-0005](../decisions/0005-wrapper-pty-como-transporte-complementar.md),
[ADR-0007](../decisions/0007-esquema-canonico-de-momentos.md).
**C4 level:** [component](../architecture/03-component.md) — the `core::normalize` module.

## Problem

All three CLIs report similar things — "I need permission", "I'm done", "I opened
a subagent" — under different event names, spellings, and fields. If each adapter
talked straight to the synthesizer, the speech policy would have to be written
three times and would drift on the first update to any of them.

## Scope

In: mapping events to canonical moments; extracting the relevant text; stamping
origin, session, and project.
Out: deciding *whether* to speak, *how*, and *in what voice* — that is
[speech-output](speech-output.md). Also out: the transport itself —
[capture-transports](capture-transports.md).

## Canonical moments

The vocabulary the rest of the system uses. Nothing beyond this crosses the
boundary:

| Moment | Meaning | Default urgency |
|---|---|---|
| `session.started` | A session began or was resumed | low |
| `session.ended` | Session ended | low |
| `turn.finished` | The agent finished answering and handed the turn back | **high** |
| `turn.failed` | The turn ended in an error | **high** |
| `decision.needed` | The agent needs a decision: permission, choice, confirmation | **critical** |
| `input.needed` | The agent is idle waiting for text from the person | **critical** |
| `subagent.started` | A subagent was created | medium |
| `subagent.finished` | A subagent finished and brought a result | medium |
| `task.created` | A task entered the list | low |
| `task.completed` | A task was completed | medium |
| `tool.started` / `tool.finished` | A tool is about to run / has run | silent by default |
| `tool.failed` | A tool failed | medium |
| `context.compacting` | History is being compacted | low |
| `message.text` | A chunk of assistant text (continuous narration) | silent by default |
| `user.returned` | The person submitted a prompt | never spoken — it **cuts off** speech in progress |
| `error` | A session error, not a tool error | **high** |

Fields on every moment:

```
momento        one of the names above
origem         "claude" | "codex" | "copilot"
transporte     "hook" | "notify" | "pty" | "stream-json" | "acp"
sessao_id      the session identifier in the originating CLI
projeto        working directory, for grouping and for picking a profile
texto          what is worth saying, already extracted (may be empty)
detalhe        the raw origin-specific map, for debugging
recebido_em    local timestamp on arrival
```

`origem` and `transporte` are deliberately separate: the same moment can arrive
by two paths and needs deduplication — see *Deduplication*.

## Map: Claude Code → moments

Source: [Hooks reference](https://code.claude.com/docs/en/hooks). Configured in
`~/.claude/settings.json`, under the `hooks` key. Payload in `snake_case` on
stdin; event name in `PascalCase` in the `hook_event_name` field.

| Claude event | Moment | Text comes from |
|---|---|---|
| `PermissionRequest` | `decision.needed` | `tool_name` + a summary of `tool_input` |
| `Notification` (`matcher: permission_prompt`) | `decision.needed` | `notification_type` |
| `Notification` (`matcher: idle_prompt`) | `input.needed` | — |
| `Elicitation` | `decision.needed` | `mcp_server_name`, `elicitation_type` |
| `Stop` | `turn.finished` | `last_assistant_message` |
| `StopFailure` | `turn.failed` | — |
| `SubagentStart` | `subagent.started` | `agent_type` |
| `SubagentStop` | `subagent.finished` | `last_assistant_message`, `agent_type` |
| `TaskCreated` | `task.created` | `task_name` |
| `TaskCompleted` | `task.completed` | `task_name` |
| `PostToolUseFailure` | `tool.failed` | `tool_name` |
| `PreToolUse` / `PostToolUse` | `tool.started` / `tool.finished` | `tool_name` |
| `MessageDisplay` | `message.text` | `message_text` |
| `UserPromptSubmit` | `user.returned` | — |
| `PreCompact` | `context.compacting` | — |
| `SessionStart` / `SessionEnd` | `session.started` / `session.ended` | `session_start_reason` / `session_end_reason` |
| `PermissionDenied` | `error` | `tool_name` |

Fields common to every payload: `session_id`, `transcript_path`, `cwd`,
`permission_mode`, `hook_event_name`.

**Watch out.** `Notification` accepts a `matcher` on `notification_type`
(`permission_prompt`, `idle_prompt`, `auth_success`, `elicitation_*`, `agent_*`)
— use the matcher instead of subscribing to everything and filtering later.

## Map: Codex CLI → moments

Sources: [Hooks system](https://deepwiki.com/openai/codex/3.11-hooks-system) and
[Advanced configuration](https://developers.openai.com/codex/config-advanced).
Configured in `~/.codex/hooks.json` (or a `[hooks]` table in
`~/.codex/config.toml`); it also accepts `.codex/` inside the repository.

| Codex event | Moment |
|---|---|
| `PermissionRequest` | `decision.needed` |
| `Stop` | `turn.finished` |
| `SubagentStart` / `SubagentStop` | `subagent.started` / `subagent.finished` |
| `PreToolUse` / `PostToolUse` | `tool.started` / `tool.finished` |
| `UserPromptSubmit` | `user.returned` |
| `PreCompact` / `PostCompact` | `context.compacting` |
| `SessionStart` / `SessionEnd` | `session.started` / `session.ended` |

Beyond hooks, Codex has the `notify` key in `config.toml`: an argv it executes at
the end of every turn, appending **one extra argument** with the event JSON.
Fields: `type` (`"agent-turn-complete"`), `last-assistant-message`,
`input-messages`, `thread-id`, `turn-id`, `cwd`. Note the **kebab-case** — it is
the only place in the system with that spelling, and `last-assistant-message` can
be missing and can be very long.

TODO: confirm on the installed version (0.147.0) which events actually fire.
There are reports that `PreToolUse`/`PostToolUse` apply only to the Bash tool,
with no hooks for file writes or MCP tools. Verify with a logging hook before
promising coverage — use the `map-cli-events` skill.

## Map: Copilot CLI → moments

Source: [Hooks reference](https://docs.github.com/en/copilot/reference/hooks-reference).
Configured in `.github/hooks/*.json` (repository) and `~/.copilot/hooks/` (user).
Structure: `{ "version": 1, "hooks": { "<event>": [ … ] } }`. Payload in
`camelCase`. Event names accept two spellings.

| Copilot event | Moment |
|---|---|
| `permissionRequest` | `decision.needed` |
| `notification` | `decision.needed` or `input.needed`, depending on `notification_type` |
| `agentStop` | `turn.finished` |
| `subagentStart` / `subagentStop` | `subagent.started` / `subagent.finished` |
| `preToolUse` / `postToolUse` | `tool.started` / `tool.finished` |
| `postToolUseFailure` | `tool.failed` |
| `errorOccurred` | `error` |
| `preCompact` | `context.compacting` |
| `userPromptSubmitted` | `user.returned` |
| `sessionStart` / `sessionEnd` | `session.started` / `session.ended` |

Copilot has no "text displayed" event. Continuous narration there comes only from
PTY or `--output-format json` — see [capture-transports](capture-transports.md).

There is also the `ask_user` tool (disabled with `--no-ask-user`): when the agent
asks something, it arrives as `preToolUse` with `toolName == "ask_user"`. Treat
that as `input.needed`, not as `tool.started`.

## Deduplication

The same occurrence can arrive twice when both hook and PTY are enabled for the
same CLI. The dedup key is `(origem, sessao_id, momento, window)`, with a
configurable tolerance window (TODO: measure; start at 1500 ms). The
highest-confidence transport wins: `hook` > `acp` > `stream-json` > `notify` >
`pty`. The loser may still enrich the winner's text when the winner arrived empty.

## Output contracts

The normalizer does not speak; it publishes on `hookd`'s internal bus. Consumers:
the speech queue ([speech-output](speech-output.md)), the GUI, and the session log.

## Alternatives considered

**One adapter per CLI talking straight to TTS.** Rejected: it triples the speech
policy and the duplication falls off the radar until it diverges.

**Using the raw payload with no canonical schema.** Rejected: it forces the GUI
and the policy to know all three dialects.

**Only Codex's `notify` and no hooks.** Rejected: `notify` only covers end of
turn, which is precisely the least urgent moment on the list.

## Test plan

TODO: write it. The minimum acceptable is a golden table — one real payload
captured for each event of each CLI, versioned under
`crates/core/tests/fixtures/`, and a test asserting the resulting moment. A
captured payload is redacted before becoming a fixture: no home paths, no client
names, no tokens.

## Open questions

- Real coverage of Codex 0.147.0's tool hooks (above).
- Claude has ~30 events; not all are mapped here. Still to decide what to do with
  `TeammateIdle`, `FileChanged`, `WorktreeCreate/Remove`, `ConfigChange`,
  `InstructionsLoaded`, `ElicitationResult`, `PostToolBatch`,
  `UserPromptExpansion`, `Setup`, `CwdChanged`, `DirectoryAdded`.
- How to version the map when a CLI renames an event without warning. Likely
  answer: an unknown moment becomes `error` with the raw name, and `cvb doctor`
  complains.
