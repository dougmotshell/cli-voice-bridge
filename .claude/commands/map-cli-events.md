<!-- managed-by:cli-voice-bridge/sync-ai-surfaces — do not edit by hand -->
---
description: Verifica empiricamente quais eventos de hook um CLI de IA dispara na versão instalada e atualiza docs/specs/event-normalization.md. Use após atualizar Claude Code, Codex CLI ou Copilot CLI, ou quando um momento parar de chegar.
---
<!-- fonte: skills/map-cli-events/SKILL.md -->

# Mapear os eventos de um CLI

A documentação diz o que deveria existir. Isto descobre o que dispara.

## Passos

1. **Versão instalada, primeiro.** `claude --version`, `codex --version`,
   `copilot --version`. Anote — o mapa vale para uma versão, não para sempre.

2. **Faça um backup da configuração de hooks** antes de tocar nela:
   `~/.claude/settings.json`, `~/.codex/hooks.json`, `~/.copilot/hooks/`.
   Esta máquina tem hooks de terceiros (`rtk`) no Claude e no Codex. Você vai
   devolver tudo como estava.

3. **Instale um hook de log temporário** — um por evento candidato — que só
   despeja o payload cru num arquivo, com o nome do evento, e sai com código 0.
   Nada que possa bloquear ou atrasar.

4. **Force cada evento** numa sessão curta e descartável: peça uma permissão,
   deixe terminar um turno, dispare um subagente, provoque uma falha de
   ferramenta. Anote o que chegou e, tão importante quanto, **o que não chegou**.

5. **Redija antes de guardar qualquer coisa.** Payload real tem caminho de casa,
   nome de projeto e às vezes trecho de código. Substitua por marcador antes de
   virar exemplo no spec ou fixture em `crates/core/tests/fixtures/`.

6. **Atualize `docs/specs/event-normalization.md`**: o mapa evento → momento, os
   nomes de campo confirmados, e a versão em que você verificou. O que ficou sem
   verificar é marcado como tal — linha errada custa mais caro que linha ausente.

7. **Remova os hooks de log e confirme que removeu.** Compare com o backup.

## Cuidados específicos

- **Codex** guarda um `trusted_hash` do comando de hook no `config.toml`: mudar o
  comando exige confirmar numa sessão, senão o hook fica inerte e você conclui,
  erradamente, que o evento não dispara.
- **Copilot** aceita duas grafias de nome de evento e usa `camelCase` no payload.
  Teste qual grafia funciona na versão instalada.
- **Claude** usa `snake_case` no payload e permite `matcher` — em `Notification`,
  filtre por `notification_type` em vez de assinar tudo.
