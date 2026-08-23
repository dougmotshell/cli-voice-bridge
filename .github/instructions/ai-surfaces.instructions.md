<!-- managed-by:cli-voice-bridge/sync-ai-surfaces — do not edit by hand -->
---
applyTo: ".claude/**,.codex/**,.agents/**,.github/prompts/**,.github/instructions/**,skills/**"
---
<!-- fonte: .claude/rules/ai-surfaces.md -->

# Superfícies de IA — fonte e projeção

**Autorados** (edite à mão): `.claude/agents/*.md`, `skills/*/SKILL.md`,
`.claude/rules/*.md`.

**Gerados** (nunca edite à mão): tudo em `.claude/skills/`, `.claude/commands/`,
`.agents/skills/`, `.codex/` e `.github/{prompts,instructions}/`. Todos carregam
o banner `managed-by:cli-voice-bridge/sync-ai-surfaces` na primeira linha.

Se você se pegou editando um arquivo com esse banner, pare: edite a fonte
correspondente (o próprio arquivo gerado diz qual, no comentário `<!-- fonte: -->`)
e rode:

```bash
python3 scripts/sync-ai-surfaces.py
python3 scripts/sync-ai-surfaces.py --check   # falha se houver divergência
```

**Frontmatter é contrato do gerador**, não decoração:
- `skills/*/SKILL.md` precisa de `name:` e `description:`, e fica abaixo de
  5.000 palavras.
- `.claude/agents/*.md` precisa de `description:`.
- `.claude/rules/*.md` precisa de `paths:` — é o que vira `applyTo:` na instrução
  do Copilot.

Sem esses campos o gerador aborta, de propósito.

**Renomear é um trabalho de duas pontas.** Renomeou uma fonte, os arquivos
gerados antigos viram órfãos: o gerador os aponta, mas não os apaga. Remova à
mão.

O contrato de comportamento do projeto mora em `AGENTS.md`. `CLAUDE.md` e
`.github/copilot-instructions.md` são adaptadores finos — se o que você quer
escrever vale para os três CLIs, ele vai no `AGENTS.md`, não no adaptador.
