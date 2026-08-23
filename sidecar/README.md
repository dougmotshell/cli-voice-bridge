# sidecar — ponte para o voice-clone

Mantém o XTTS-v2 carregado e sintetiza sob demanda para o `hookd`.

**Por que existe.** Carregar o XTTS-v2 leva cerca de 30 segundos. Um `spawn` de
`falar.py` por frase pagaria isso em cada fala, o que é inaceitável — daí um
processo de vida longa (ADR-0003).

**Por que é Python.** O motor é Python e a voz clonada é o produto. Reimplementar
em Rust não está na mesa.

## Contrato

O `voice-clone` é **dependência externa somente leitura**. Este sidecar importa
`vozclone` do venv de lá; não copia código, não edita nada lá, e o caminho vem
sempre da configuração — nunca embutido.

```bash
CVB_VOICE_CLONE=/caminho/para/voice-clone \
  /caminho/para/voice-clone/.venv/bin/python sidecar/servidor.py
```

Protocolo: uma linha JSON por requisição, uma por resposta, no socket de
`cvb_core::caminhos::endereco_sidecar()`.

```json
{"tipo": "sintetizar", "texto": "...", "voz": "douglas", "idioma": "pt-BR", "saida": "/tmp/x.wav"}
{"tipo": "ok", "caminho": "/tmp/x.wav", "duracao_s": 1.8}
```

## Estado

Esqueleto. O laço de servidor e o protocolo estão escritos; a carga do modelo é
preguiçosa e ainda não foi exercitada. TODO: cache de frases fixas, fila
própria, e o teste de fumaça de `skills/smoke-voice/SKILL.md`.
