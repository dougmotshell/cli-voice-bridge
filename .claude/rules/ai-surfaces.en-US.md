> Translation of [`ai-surfaces.md`](ai-surfaces.md), the source of truth. No
> frontmatter on purpose: only the pt-BR file carries `paths:` and is loaded as a
> rule.

# AI surfaces — source and projection

**Authored** (edit by hand): `.claude/agents/*.md`, `skills/*/SKILL.md`,
`.claude/rules/*.md`.

**Generated** (never edit by hand): everything in `.claude/skills/`,
`.claude/commands/`, `.agents/skills/`, `.codex/`, and
`.github/{prompts,instructions}/`. They all carry the
`managed-by:cli-voice-bridge/sync-ai-surfaces` banner on line 1.

If you catch yourself editing a file with that banner, stop: edit the
corresponding source (the generated file itself says which, in the
`<!-- fonte: -->` comment) and run:

```bash
python3 scripts/sync-ai-surfaces.py
python3 scripts/sync-ai-surfaces.py --check   # fails on drift
```

**Frontmatter is the generator's contract**, not decoration:
- `skills/*/SKILL.md` needs `name:` and `description:`, and stays under 5,000
  words.
- `.claude/agents/*.md` needs `description:`.
- `.claude/rules/*.md` needs `paths:` — that is what becomes `applyTo:` in the
  Copilot instruction.

Without those fields the generator aborts, on purpose.

**Translations carry no frontmatter and are ignored by the generator.** A file
named `<name>.en-US.md` or `SKILL.en-US.md` is a sibling for people to read.
Projected, it would become a second skill or rule with the same `name`; with
frontmatter, a CLI could load it as a duplicate definition.

**Renaming is a two-ended job.** Rename a source and the old generated files
become orphans: the generator points them out but does not delete them. Remove
them by hand.

This project's behavior contract lives in `AGENTS.md`. `CLAUDE.md` and
`.github/copilot-instructions.md` are thin adapters — if what you want to write
holds for all three CLIs, it belongs in `AGENTS.md`, not in an adapter.
